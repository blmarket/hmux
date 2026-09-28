use crate::src::arguments::{
    args_count, args_flag_values, args_flags, args_get, args_print_cstring, args_string,
};
use crate::src::cfg::{cfg_add_cause, cfg_finished};
use crate::src::cmd::find::{
    cmd_find_clear_state, cmd_find_client, cmd_find_copy_state, cmd_find_from_client,
    cmd_find_target, cmd_find_valid_state,
};
use crate::src::cmd::{
    cmd_get_args_mut, cmd_get_entry, cmd_get_group, cmd_get_source, cmd_print_cstring,
};
use crate::src::control::{control_write, control_write_guard};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_identity, event_payload_set_string,
    event_payload_set_target,
};
use crate::src::ffi::libc::{__ctype_toupper_loc, getpwuid, getuid, time};
use crate::src::file::{file_cancel_cmdq_wait, file_error};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::{xformat, xformat_with};
use crate::src::format::{format_add, format_add_cstr, format_create_owned, format_owner_ptr, format_merge};
use crate::src::key_string::key_string_format;
use crate::src::log::{fatalx, log_cstr, log_debug, log_get_level};
use crate::src::proc::proc_get_peer_uid;
use crate::src::reactor::{evbuffer_add_formatted, evbuffer_new};
use crate::src::server::server_add_message;
use crate::src::server_client::{server_client_print, server_client_unref_owned};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__uid_t, uid_t};
use crate::src::shared::account::passwd;
use crate::src::shared::arguments::args;
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
use std::ffi::{CStr, CString};

pub const CMDQ_CALLBACK: cmdq_type = 1;
pub const CMDQ_COMMAND: cmdq_type = 0;

unsafe fn cmdq_new_named_item(label: Option<&CStr>) -> std::rc::Rc<std::cell::UnsafeCell<cmdq_item>> {
    let owner = std::rc::Rc::new(std::cell::UnsafeCell::new(cmdq_item::empty()));
    let item = owner.get();
    (*item).observer = std::rc::Rc::downgrade(&owner);
    let label = label.map_or(b"(null)".as_slice(), CStr::to_bytes);
    let address = format!("{item:p}");
    let mut bytes = Vec::with_capacity(label.len() + address.len() + 3);
    bytes.push(b'[');
    bytes.extend_from_slice(label);
    bytes.push(b'/');
    bytes.extend_from_slice(address.as_bytes());
    bytes.push(b']');
    (*item).name = Some(CString::new(bytes).expect("queue item label has no NUL"));

    owner
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

unsafe fn cmdq_cancel_unfired_data(owner: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    let item = owner.get();
    if (*item).flags & CMDQ_FIRED != 0 {
        return;
    }
    if let Some(cancel) = (*item).cancel_data.take() {
        cancel();
    }
}

/// The file remains live until its terminal event, including when a local
/// file operation schedules immediate completion. The queue item only borrows
/// it through a weak handle while the command is waiting.
pub(crate) fn cmdq_set_wait_file(
    item: &mut cmdq_item,
    file: &std::rc::Rc<std::cell::UnsafeCell<client_file>>,
) {
    assert!(
        std::rc::Weak::ptr_eq(&item.wait_file, &std::rc::Weak::new()),
        "queue item already has a file wait"
    );
    item.wait_file = std::rc::Rc::downgrade(file);
}

pub(crate) fn cmdq_clear_wait_file(
    item: &mut cmdq_item,
    file: &std::rc::Weak<std::cell::UnsafeCell<client_file>>,
) {
    if item.wait_file.ptr_eq(file) {
        item.wait_file = std::rc::Weak::new();
    }
}

/// A dead client cannot resume a file-backed waiting command. Cancel its
/// callback data first, then remove the waiting item and its queued suffix.
/// Other wait families need their own cancellation before they can be drained.
pub(crate) unsafe fn cmdq_abort_file_wait(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let queue = cmdq_get(Some(owner));
    let Some(first) = (*queue).first() else { return };
    if (*first.get()).flags & CMDQ_WAITING == 0 { return; }
    let Some(file) = (*first.get()).wait_file.upgrade() else { return };
    file_cancel_cmdq_wait(&file);
    drop(first);
    (*queue).item = std::rc::Weak::new();
    while let Some(item) = (*queue).list.pop_front() {
        cmdq_remove(item);
    }
}

fn cmdq_name(c: Option<&client>) -> CString {
    let Some(c) = c else {
        return c"<global>".to_owned();
    };
    format_message_with(|out| {
        if let Some(name) = c.name.as_ref() {
            out.write_all(b"<")?;
            out.write_all(name.as_bytes())?;
            out.write_all(b">")
        } else {
            write!(out, "<{:p}>", c)
        }
    })
}
unsafe fn cmdq_get(owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>) -> *mut cmdq_list {
    static mut GLOBAL_QUEUE: Option<Box<cmdq_list>> = None;
    match owner {
        None => &mut **(&mut *(&raw mut GLOBAL_QUEUE)).get_or_insert_with(cmdq_new),
        Some(owner) => (*owner.get()).queue.as_deref_mut().expect("client command queue"),
    }
}

pub fn cmdq_new() -> Box<cmdq_list> {
    Box::new(cmdq_list {
        item: std::rc::Weak::new(),
        list: std::collections::VecDeque::new(),
    })
}

impl Drop for cmdq_list {
    fn drop(&mut self) {
        if !self.list.is_empty() {
            unsafe { fatalx(|out| out.write_all(b"queue not empty")) };
        }
    }
}

pub fn cmdq_get_name(item: &cmdq_item) -> Option<&std::ffi::CStr> {
    item.name.as_deref()
}
pub fn cmdq_get_cmd(item: &cmdq_item) -> *mut cmd {
    item.cmd_ptr()
}
pub fn cmdq_get_client(item_handle: Option<&cmdq_item>) -> Option<std::rc::Rc<std::cell::UnsafeCell<client>>> {
    item_handle.and_then(|item| item.client.upgrade())
}
pub fn cmdq_get_target_client(item_handle: Option<&cmdq_item>) -> Option<std::rc::Rc<std::cell::UnsafeCell<client>>> {
    item_handle.and_then(|item| item.target_client.upgrade())
}
pub fn cmdq_get_state(item: &cmdq_item) -> Option<&std::rc::Rc<cmdq_state>> {
    item.state.as_ref()
}
pub fn cmdq_get_target(item: &cmdq_item) -> &cmd_find_state {
    &item.target
}
pub fn cmdq_get_target_mut(item: &mut cmdq_item) -> &mut cmd_find_state {
    &mut item.target
}
pub fn cmdq_get_source(item: &cmdq_item) -> &cmd_find_state {
    &item.source
}
pub fn cmdq_get_source_mut(item: &mut cmdq_item) -> &mut cmd_find_state {
    &mut item.source
}
/// Event metadata is read from a snapshot, never through a pointer into shared state.
pub fn cmdq_get_event(item: &cmdq_item) -> key_event {
    cmdq_get_state(item).expect("command queue state").event.metadata_snapshot()
}
/// Retain the state; callers borrow its current target only for each operation.
pub fn cmdq_get_state_owned(item: &cmdq_item) -> std::rc::Rc<cmdq_state> {
    cmdq_get_state(item).expect("command queue state").clone()
}
pub fn cmdq_get_flags(item: &cmdq_item) -> ::core::ffi::c_int {
    cmdq_get_state(item).expect("command queue state").flags
}
pub unsafe fn cmdq_new_state(
    mut current: *mut cmd_find_state,
    mut event: *mut key_event,
    mut flags: ::core::ffi::c_int,
) -> std::rc::Rc<cmdq_state> {
    let snapshot = if event.is_null() {
        key_event {
            client: std::rc::Weak::new(),
            key: KEYC_NONE as key_code,
            m: Default::default(),
            bytes: None,
        }
    } else {
        (*event).metadata_snapshot()
    };
    let mut target = cmd_find_state::default();
    if !current.is_null() && cmd_find_valid_state(&*current) != 0 {
        cmd_find_copy_state(&mut target, current);
    } else {
        cmd_find_clear_state(&mut target, 0);
    }
    std::rc::Rc::new(cmdq_state {
        flags,
        formats: Default::default(),
        event: snapshot,
        current: std::cell::RefCell::new(target),
    })
}
pub unsafe fn cmdq_copy_state(state: &cmdq_state, current: *mut cmd_find_state) -> std::rc::Rc<cmdq_state> {
    let mut event = state.event.metadata_snapshot();
    if !current.is_null() {
        return cmdq_new_state(current, &mut event, state.flags);
    }
    let mut current = state.current_snapshot();
    cmdq_new_state(&mut current, &mut event, state.flags)
}
/// Copy a literal key and value into the command queue state's formats.
pub unsafe fn cmdq_add_format(state: &cmdq_state, key: &CStr, value: &CStr) {
    let mut formats = state.formats.borrow_mut();
    if formats.is_none() {
        *formats = Some(format_create_owned(
            None,
            None,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        ));
    }
    format_add_cstr(format_owner_ptr(&mut *formats), key, value);
}
pub unsafe fn cmdq_add_formats(state: &cmdq_state, mut ft: *mut format_tree) {
    let mut formats = state.formats.borrow_mut();
    if formats.is_none() {
        *formats = Some(format_create_owned(
            None,
            None,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        ));
    }
    format_merge(format_owner_ptr(&mut *formats), ft);
}
pub unsafe fn cmdq_merge_formats(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>, mut ft: *mut format_tree) {
    let item = item_handle.get();
    if !(*item).cmd_ptr().is_null() {
        let entry = cmd_get_entry(&*(*item).cmd_ptr());
        format_add(
            ft,
            b"command\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, entry.name.as_ptr()),
        );
    }
    let state = cmdq_get_state(&*item).expect("command queue state").clone();
    let mut formats = state.formats.borrow_mut();
    if formats.is_some() {
        format_merge(ft, format_owner_ptr(&mut formats));
    }
}
/// Consume a detached chain, moving its owners into the queue in order.
pub unsafe fn cmdq_append(
    owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    item: std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> std::rc::Weak<std::cell::UnsafeCell<cmdq_item>> {
    let queue = cmdq_get(owner);
    let mut remaining = Some(item);
    let mut last = std::rc::Weak::new();
    while let Some(owner_item) = remaining {
        let item = owner_item.get();
        assert!((*item).queue.is_null() && !(*item).removed, "detached command item");
        remaining = (*item).next.take();
        (*item).client_owner = owner.cloned();
        (*item).client = owner.map(std::rc::Rc::downgrade).unwrap_or_default();
        (*item).queue = queue;
        last = std::rc::Rc::downgrade(&owner_item);
        (*queue).list.push_back(owner_item);
        log_debug(format_args!(
            "{} {}: {}",
            "cmdq_append",
            crate::src::log::log_bytes(
                cmdq_name(owner.map(|owner| &*owner.get())).as_bytes()
            ),
            log_cstr(
                (((*item).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
    }
    last
}

/// Consume a detached chain and return the final queued insertion anchor.
pub unsafe fn cmdq_insert_after(
    after: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    item: std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> std::rc::Weak<std::cell::UnsafeCell<cmdq_item>> {
    assert!(!(*after.get()).removed && !(*after.get()).queue.is_null(), "queued insertion anchor");
    let c_owner = (*after.get()).client.upgrade();
    let queue = (*after.get()).queue;
    let mut position = (*queue).position(after) + 1;
    let mut previous = after.get();
    let mut remaining = Some(item);
    let mut last = std::rc::Weak::new();
    while let Some(owner_item) = remaining {
        let item = owner_item.get();
        assert!((*item).queue.is_null() && !(*item).removed, "detached command item");
        remaining = (*item).next.take();
        (*item).client_owner = c_owner.clone();
        (*item).client = c_owner.as_ref().map(std::rc::Rc::downgrade).unwrap_or_default();
        (*item).queue = queue;
        last = std::rc::Rc::downgrade(&owner_item);
        (*queue).list.insert(position, owner_item);
        position += 1;
        log_debug(format_args!(
            "{} {}: {} after {}",
            "cmdq_insert_after",
            crate::src::log::log_bytes(
                cmdq_name(c_owner.as_ref().map(|owner| &*owner.get())).as_bytes()
            ),
            log_cstr(
                (((*item).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr(
                (((*previous).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        previous = item;
    }
    last
}
pub unsafe fn cmdq_insert_hook(
    _s_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut current: *mut cmd_find_state,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let item = item_handle.get();
    let mut _s = _s_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut cmd: *mut cmd = (*item).cmd_ptr();
    let mut args_0: *mut args = cmd_get_args_mut(&mut *cmd).map_or(std::ptr::null_mut(), |args| args);
    let mut tmp: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cmdq_get_state(&*item).expect("command queue state").flags & CMDQ_STATE_NOHOOKS != 0 {
        return;
    }
    let name = format_message_with(write);
    let mut ep = event_payload_create();
    if !current.is_null() {
        event_payload_set_target(&mut *ep, &*current);
    }
    event_payload_set_identity(
        &mut *ep,
        b"_cmdq_item\0" as *const u8 as *const ::core::ffi::c_char,
        crate::src::shared::events::EventPayloadIdentity::QueueItem((*item).observer.clone()),
    );
    let arguments = args_print_cstring(&*args_0);
    event_payload_set_string(
        &mut *ep,
        b"arguments\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, arguments.as_ptr()),
    );
    i = 0 as u_int;
    while i < args_count(args_0) {
        xformat(&mut tmp, format_args!("argument_{}", i as u32));
        event_payload_set_string(&mut *ep, &raw mut tmp as *mut ::core::ffi::c_char, |out| {
            write_cstr(out, args_string(&mut *(args_0), i).map_or(std::ptr::null(), |value| value.as_ptr()))
        });
        i = i.wrapping_add(1);
    }
    for flag in args_flags(&*args_0) {
        value = args_get(&*(args_0), flag as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
        xformat_with(&mut tmp, |out| {
            out.write_all(b"flag_")?;
            out.write_all(&[flag as u8])
        });
        if value.is_null() {
            event_payload_set_string(&mut *ep, &raw mut tmp as *mut ::core::ffi::c_char, |out| {
                out.write_all(b"1")
            });
        } else {
            event_payload_set_string(&mut *ep, &raw mut tmp as *mut ::core::ffi::c_char, |out| {
                write_cstr(out, value)
            });
        }
        i = 0 as u_int;
        for av in args_flag_values(&*args_0, flag as u_char) {
            xformat_with(&mut tmp, |out| {
                out.write_all(b"flag_")?;
                out.write_all(&[flag as u8])?;
                write!(out, "_{}", i)
            });
            event_payload_set_string(&mut *ep, &raw mut tmp as *mut ::core::ffi::c_char, |out| {
                write_cstr(out, av.string_ptr())
            });
            i = i.wrapping_add(1);
        }
    }
    events_fire(name.as_ptr(), ep);
}
pub unsafe fn cmdq_continue(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    let item = item_handle.get();
    assert!(!(*item).removed, "cannot resume a removed command item");
    (*item).flags &= !CMDQ_WAITING;
}
// The caller transfers the queue owner after unlinking it and releasing guards.
unsafe fn cmdq_remove(item_handle: std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    assert_eq!(std::rc::Rc::strong_count(&item_handle), 1, "command item removal requires its sole strong owner");
    let item = item_handle.get();
    assert!(!(*item).removed, "command item already removed");
    (*item).removed = true;
    assert!(
        std::rc::Weak::ptr_eq(&(*item).wait_file, &std::rc::Weak::new()),
        "file wait must finish or cancel before queue item removal"
    );
    cmdq_cancel_unfired_data(&item_handle);
    if let Some(client) = (*item).client_owner.take() {
        server_client_unref_owned(client);
    }
    drop((*item).cmdlist.take());
    drop((*item).state.take());
    let queue = (*item).queue;
    if (*queue).item.ptr_eq(&(*item).observer) {
        (*queue).item = std::rc::Weak::new();
    }
    (*item).queue = std::ptr::null_mut();
    // Release resources explicitly before destroying the item.
    drop((*item).cb.take());
    drop((*item).cancel_data.take());
    drop(item_handle);
}
unsafe fn cmdq_remove_group(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    let item = item_handle.get();
    if (*item).group == 0 as u_int {
        return;
    }
    let queue = (*item).queue;
    let mut position = (*queue).position(item_handle) + 1;
    while let Some(owner) = (*queue).list.get(position) {
        let this = owner.get();
        if (*this).group == (*item).group {
            let owner = (*queue).list.remove(position).expect("queued group item");
            cmdq_remove(owner);
        } else {
            position += 1;
        }
    }
}
#[must_use = "enqueue the detached command chain"]
pub unsafe fn cmdq_get_command(
    commands: &std::rc::Rc<std::cell::RefCell<cmd_list>>,
    state: Option<&std::rc::Rc<cmdq_state>>,
) -> std::rc::Rc<std::cell::UnsafeCell<cmdq_item>> {
    let cmdlist = commands.borrow();
    if cmdlist.list.is_empty() {
        return cmdq_get_callback_owned(c"cmdq_empty_command".as_ptr(), Some(Box::new(|_| CMD_RETURN_NORMAL)));
    }
    let state = state.cloned().unwrap_or_else(|| cmdq_new_state(
        std::ptr::null_mut(), std::ptr::null_mut(), 0,
    ));
    let mut remaining = None;
    for command in cmdlist.list.iter().rev() {
        let cmd = command.as_ptr().cast_mut();
        let owner = cmdq_new_named_item(Some(cmd_get_entry(&*cmd).name));
        let item = owner.get();
        (*item).type_0 = CMDQ_COMMAND;
        (*item).group = cmd_get_group(cmd);
        (*item).state = Some(state.clone());
        (*item).cmd = command.downgrade();
        (*item).cmdlist = Some(commands.clone());
        (*item).next = remaining;
        remaining = Some(owner);
    }
    remaining.expect("nonempty command list")
}
unsafe fn cmdq_find_flag(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut fs: *mut cmd_find_state,
    flag: &cmd_entry_flag,
) -> cmd_retval {
    let item = item_handle.get();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if flag.flag as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let target_client = cmdq_get_target_client((item).as_ref());
        let target_client_ptr = target_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
        cmd_find_from_client(fs, (target_client_ptr).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), 0 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    value = args_get(&*(cmd_get_args_mut(&mut *(*item).cmd_ptr()).map_or(std::ptr::null_mut(), |args| args)), flag.flag as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if cmd_find_target(fs, Some(item_handle), value, flag.type_0, flag.flags) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, 0 as ::core::ffi::c_int);
        return CMD_RETURN_ERROR;
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmdq_add_message(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let state = cmdq_get_state(&*item).expect("command queue state").clone();
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let tmp = cmd_print_cstring(&*(*item).cmd_ptr());
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
        if !(*c).session_handle().is_none()
            && state.event.key != KEYC_NONE as ::core::ffi::c_ulong as key_code
        {
            let key = key_string_format(state.event.key, false);
            server_add_message(|out| {
                write_cstr(
                    out,
                    ((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )?;
                write_cstr(out, user.as_ptr())?;
                out.write_all(b" key ")?;
                write_cstr(out, key.as_ptr())?;
                out.write_all(b": ")?;
                write_cstr(out, tmp.as_ptr())
            });
        } else {
            server_add_message(|out| {
                write_cstr(
                    out,
                    ((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )?;
                write_cstr(out, user.as_ptr())?;
                out.write_all(b" command: ")?;
                write_cstr(out, tmp.as_ptr())
            });
        }
    } else {
        server_add_message(|out| {
            out.write_all(b"command: ")?;
            write_cstr(out, tmp.as_ptr())
        });
    }
}
unsafe fn cmdq_fire_command(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let mut current_block: u64;
    let saved_client = cmdq_get_client((item).as_ref());
    let saved_client_ptr = saved_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let name = cmdq_name(saved_client.as_ref().map(|owner| &*owner.get()));
    let state = cmdq_get_state(&*item).expect("command queue state").clone();
    let mut cmd: *mut cmd = (*item).cmd_ptr();
    let mut args: *mut args = cmd_get_args_mut(&mut *cmd).map_or(std::ptr::null_mut(), |args| args);
    let entry = cmd_get_entry(&*cmd);
    let mut tc = None;
    let saved = (*item).client.clone();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fsp: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut flags: ::core::ffi::c_int = 0;
    let mut quiet: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cfg_finished != 0 {
        cmdq_add_message(item_handle);
    }
    if log_get_level() > 1 as ::core::ffi::c_int {
        let tmp = cmd_print_cstring(&*cmd);
        log_debug(format_args!(
            "{} {}: ({}) {}",
            "cmdq_fire_command",
            crate::src::log::log_bytes(name.as_bytes()),
            ((*item).group) as u32,
            crate::src::log::log_bytes(tmp.as_bytes())
        ));
    }
    flags = (state.flags & CMDQ_STATE_CONTROL != 0) as ::core::ffi::c_int;
    cmdq_guard(
        item_handle,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        flags,
    );
    if (*item).client.upgrade().is_none() {
        let context = cmd_find_client(
            Some(item_handle),
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
        (*item).client = context.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    }
    let execution_client = cmdq_get_client((item).as_ref());
    let execution_client_ptr = execution_client.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    if entry.flags & CMD_CLIENT_CANFAIL != 0 {
        quiet = 1 as ::core::ffi::c_int;
    }
    if entry.flags & CMD_CLIENT_CFLAG != 0 {
        tc = cmd_find_client(Some(item_handle), args_get(&*(args), 'c' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()), quiet);
        if tc.is_none() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else if entry.flags & CMD_CLIENT_TFLAG != 0 {
        tc = cmd_find_client(Some(item_handle), args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()), quiet);
        if tc.is_none() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else {
        tc = cmd_find_client(
            Some(item_handle),
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
        current_block = 18317007320854588510;
    }
    match current_block {
        18317007320854588510 => {
            (*item).target_client = tc.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
            let _target_client = cmdq_get_target_client((item).as_ref());
            retval = cmdq_find_flag(item_handle, &raw mut (*item).source, &entry.source);
            if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                retval = cmdq_find_flag(item_handle, &raw mut (*item).target, &entry.target);
                if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                    retval = entry.exec.expect("non-null function pointer")(cmd, item_handle);
                    assert!(!(*item).removed, "executing command item removed during dispatch");
                    if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                        if entry.flags & CMD_AFTERHOOK != 0 {
                            if cmd_find_valid_state(&(*item).target) != 0 {
                                fsp = &raw mut (*item).target;
                                current_block = 8704759739624374314;
                            } else if cmd_find_valid_state(&state.current.borrow()) != 0 {
                                fs = state.current_snapshot();
                                fsp = &mut fs;
                                current_block = 8704759739624374314;
                            } else if cmd_find_from_client(
                                &raw mut fs,
                                (execution_client_ptr).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
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
                                    cmdq_insert_hook(((*fsp).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), item_handle, fsp, |out| {
                                        out.write_all(b"after-")?;
                                        write_cstr(out, entry.name.as_ptr())
                                    });
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
        if cmd_find_valid_state(&(*item).target) != 0 {
            fsp = &raw mut (*item).target;
        } else if cmd_find_valid_state(&state.current.borrow()) != 0 {
            fs = state.current_snapshot();
            fsp = &mut fs;
        } else if cmd_find_from_client(&raw mut fs, (saved_client_ptr).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), 0 as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
        {
            fsp = &raw mut fs;
        }
        cmdq_insert_hook(
            (if !fsp.is_null() {
                (*fsp).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())
            } else {
                ::core::ptr::null_mut::<session>()
            }).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
            item_handle,
            fsp,
            |out| out.write_all(b"command-error"),
        );
        cmdq_guard(
            item_handle,
            b"error\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    } else {
        cmdq_guard(
            item_handle,
            b"end\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    }
    return retval;
}
#[must_use = "enqueue the detached command chain"]
pub unsafe fn cmdq_get_callback_owned(
    mut name: *const ::core::ffi::c_char,
    mut cb: cmdq_cb,
) -> std::rc::Rc<std::cell::UnsafeCell<cmdq_item>> {
    let owner = cmdq_new_named_item(if name.is_null() {
        None
    } else {
        Some(CStr::from_ptr(name))
    });
    let item = owner.get();
    (*item).type_0 = CMDQ_CALLBACK;
    (*item).group = 0 as u_int;
    (*item).state = Some(cmdq_new_state(
        ::core::ptr::null_mut::<cmd_find_state>(),
        ::core::ptr::null_mut::<key_event>(),
        0 as ::core::ffi::c_int,
    ));
    (*item).cb = cb;
    owner
}

#[must_use = "enqueue the detached command chain"]
pub unsafe fn cmdq_get_error(mut error: *const ::core::ffi::c_char) -> std::rc::Rc<std::cell::UnsafeCell<cmdq_item>> {
    let error = CStr::from_ptr(error).to_owned();
    cmdq_get_callback_owned(
        b"cmdq_error_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(Box::new(move |item| unsafe {
            cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
            CMD_RETURN_NORMAL
        })),
    )
}
unsafe fn cmdq_fire_callback(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    (*item).flags |= CMDQ_FIRED;
    return (*item).cb.take().expect("non-null queue callback")(
        item_handle,
    );
}
pub unsafe fn cmdq_next(owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>) -> u_int {
    let queue = cmdq_get(owner);
    let name = cmdq_name(owner.map(|owner| &*owner.get()));
    let mut items = 0;
    log_debug(format_args!("cmdq_next {}: enter", crate::src::log::log_bytes(name.as_bytes())));
    static mut number: u_int = 0;
    while let Some(item_owner) = (*queue).first() {
        let item = item_owner.get();
        (*queue).item = std::rc::Rc::downgrade(&item_owner);
        assert!(!(*item).removed, "queued command item");
        log_debug(format_args!("cmdq_next {}: {} ({}), flags {:x}",
            crate::src::log::log_bytes(name.as_bytes()),
            crate::src::log::log_bytes((*item).name.as_deref().unwrap_or(c"(null)").to_bytes()),
            (*item).type_0, (*item).flags));
        if (*item).flags & CMDQ_WAITING != 0 { return items; }
        if (*item).flags & CMDQ_FIRED == 0 {
            (*item).time = time(std::ptr::null_mut());
            number = number.wrapping_add(1);
            (*item).number = number;
            let retval = match (*item).type_0 {
                CMDQ_COMMAND => cmdq_fire_command(&item_owner),
                CMDQ_CALLBACK => cmdq_fire_callback(&item_owner),
                _ => CMD_RETURN_ERROR,
            };
            // Dispatch may insert items, but cannot remove its own active item.
            assert!(!(*item).removed, "executing command item removed during dispatch");
            if retval == CMD_RETURN_ERROR && (*item).type_0 == CMDQ_COMMAND {
                cmdq_remove_group(&item_owner);
            }
            (*item).flags |= CMDQ_FIRED;
            if retval == CMD_RETURN_WAIT {
                (*item).flags |= CMDQ_WAITING;
                return items;
            }
            items += 1;
        }
        drop(item_owner);
        let owner = (*queue).list.pop_front().expect("executed queue item");
        cmdq_remove(owner);
    }
    (*queue).item = std::rc::Weak::new();
    items
}

pub unsafe fn cmdq_running() -> std::rc::Weak<std::cell::UnsafeCell<cmdq_item>> {
    let queue = cmdq_get(None);
    let Some(owner) = (*queue).item.upgrade() else { return std::rc::Weak::new() };
    if (*owner.get()).removed || (*owner.get()).flags & CMDQ_WAITING != 0 {
        return std::rc::Weak::new();
    }
    std::rc::Rc::downgrade(&owner)
}
pub unsafe fn cmdq_guard(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut guard: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut t: ::core::ffi::c_long = (*item).time as ::core::ffi::c_long;
    let mut number: u_int = (*item).number;
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_write_guard(&(*(c)).observer.upgrade().expect("live client"), guard, t, number, flags);
    }
}
pub unsafe fn cmdq_print_data(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>, mut evb: &mut evbuffer) {
    let item = item_handle.get();
    let client = cmdq_get_client((item).as_ref());
    server_client_print(client.as_ref(), 1 as ::core::ffi::c_int, evb);
}
pub unsafe fn cmdq_print(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut evb = evbuffer_new();
    evbuffer_add_formatted(&mut *evb, write);
    cmdq_print_data(item_handle, &mut *evb);
}
pub unsafe fn cmdq_error(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut cmd: *mut cmd = (*item).cmd_ptr();
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    let mut msg = format_message_with(write);
    log_debug(format_args!(
        "{}: {}",
        "cmdq_error",
        crate::src::log::log_bytes(msg.as_bytes())
    ));
    if c.is_null() {
        let (source, source_line) = cmd_get_source(&*cmd);
        file = source.map_or(std::ptr::null(), |file| file.as_ptr());
        line = source_line;
        if cfg_finished == 0 {
            if !file.is_null() {
                cfg_add_cause(|out| {
                    write_cstr(out, file)?;
                    write!(out, ":{}: ", (line) as u32)?;
                    write_cstr(out, msg.as_ptr())
                });
            } else {
                cfg_add_cause(|out| write_cstr(out, msg.as_ptr()));
            }
        } else if !file.is_null() {
            server_add_message(|out| {
                out.write_all(b"message: ")?;
                write_cstr(out, file)?;
                write!(out, ":{}: ", (line) as u32)?;
                write_cstr(out, msg.as_ptr())
            });
        } else {
            server_add_message(|out| {
                out.write_all(b"message: ")?;
                write_cstr(out, msg.as_ptr())
            });
        }
    } else if (*c).session_handle().is_none() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        server_add_message(|out| {
            write_cstr(
                out,
                ((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )?;
            out.write_all(b" message: ")?;
            write_cstr(out, msg.as_ptr())
        });
        if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
            msg = utf8_sanitize_cstring(msg.as_c_str());
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(&(*(c)).observer.upgrade().expect("live client"), |out| write_cstr(out, msg.as_ptr()));
        } else {
            file_error(c_owner.as_ref(), |out| {
                write_cstr(out, msg.as_ptr())?;
                out.write_all(b"\n")
            });
        }
        (*c).retval = 1 as ::core::ffi::c_int;
    } else {
        let mut bytes = msg.into_bytes_with_nul();
        bytes[0] = *(*__ctype_toupper_loc()).offset(bytes[0] as isize) as u8;
        // The bytes came from a CString; toupper maps NUL to NUL and a
        // non-NUL byte to a non-NUL byte, so the terminator stays at the end.
        msg = CString::from_vec_with_nul_unchecked(bytes);
        status_message_set(
            (c).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| write_cstr(out, msg.as_ptr()),
        );
    }
}

#[cfg(test)]
mod client_observer_tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn client_accessors_retain_upgrades_and_handle_expired_observers() {
        unsafe {
            let mut queue = cmdq_new();
            let item_owner = cmdq_get_callback_owned(c"accessor test".as_ptr(), None);
            let item = &mut *item_owner.get();
            item.queue = &mut *queue;
            assert!(cmdq_get_client(Some(&item)).is_none());
            assert!(cmdq_get_target_client(Some(&item)).is_none());

            let owner = client::new();
            item.client = Rc::downgrade(&owner);
            item.target_client = Rc::downgrade(&owner);
            assert_eq!(Rc::strong_count(&owner), 1);
            let context = cmdq_get_client(Some(&item)).unwrap();
            let target = cmdq_get_target_client(Some(&item)).unwrap();
            assert!(Rc::ptr_eq(&owner, &context));
            assert!(Rc::ptr_eq(&owner, &target));
            drop(owner);
            drop(context);
            assert!(item.client.upgrade().is_some());
            drop(target);
            assert!(cmdq_get_client(Some(&item)).is_none());
            assert!(cmdq_get_target_client(Some(&item)).is_none());
            cmdq_remove(item_owner);
        }
    }

    #[test]
    fn retained_client_queue_resumes_after_wait_and_drains_callbacks() {
        unsafe {
            let owner = client::new();
            (*owner.get()).queue = Some(cmdq_new());
            let observer = Rc::downgrade(&owner);
            let calls = Rc::new(std::cell::Cell::new(0));
            let waiting_allocation = cmdq_get_callback_owned(c"wait".as_ptr(), Some(Box::new(|_| CMD_RETURN_WAIT)));
            let waiting = waiting_allocation.get();
            let seen = calls.clone();
            let expected = observer.clone();
            let following_allocation = cmdq_get_callback_owned(c"following".as_ptr(), Some(Box::new(move |item| {
                assert!(Rc::ptr_eq(&cmdq_get_client(Some(&*item.get())).unwrap(), &expected.upgrade().unwrap()));
                seen.set(seen.get() + 1);
                CMD_RETURN_NORMAL
            })));
            (*waiting).next = Some(following_allocation);
            cmdq_append(Some(&owner), waiting_allocation);
            assert_eq!(cmdq_next(Some(&owner)), 0);
            assert_eq!(calls.get(), 0);
            assert_eq!(cmdq_next(Some(&owner)), 0);
            cmdq_continue(&(*(waiting)).observer.upgrade().expect("live command queue item"));
            assert_eq!(cmdq_next(Some(&owner)), 1);
            assert_eq!(calls.get(), 1);
            assert!((*owner.get()).queue.as_ref().unwrap().list.is_empty());
            drop(owner);
            crate::src::reactor::event_loop();
            assert!(observer.upgrade().is_none());
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn append_and_insert_retain_client_for_every_queued_item() {
        unsafe {
            let owner = client::new();
            (*owner.get()).queue = Some(cmdq_new());
            let observer = Rc::downgrade(&owner);
            let first_allocation = cmdq_get_callback_owned(c"first".as_ptr(), None);
            let first = first_allocation.get();
            let second_allocation = cmdq_get_callback_owned(c"second".as_ptr(), None);
            let second = second_allocation.get();
            (*first).next = Some(second_allocation);
            assert_eq!(cmdq_append(Some(&owner), first_allocation).upgrade().unwrap().get(), second);
            let inserted_allocation = cmdq_get_callback_owned(c"inserted".as_ptr(), None);
            let inserted = inserted_allocation.get();
            assert_eq!(cmdq_insert_after(&(*first).observer.upgrade().unwrap(), inserted_allocation).upgrade().unwrap().get(), inserted);
            assert_eq!(Rc::strong_count(&owner), 4);
            assert!(Rc::ptr_eq(&cmdq_get_client(inserted.as_ref()).unwrap(), &owner));
            assert!(Rc::ptr_eq(&cmdq_get_client(first.as_ref()).unwrap(), &owner));
            assert!(Rc::ptr_eq(&cmdq_get_client(second.as_ref()).unwrap(), &owner));
            assert!((*first).next.is_none());
            drop(owner);
            let retained = observer.upgrade().expect("queued items retain their client");
            let queue = (*retained.get()).queue.as_deref_mut().unwrap() as *mut cmdq_list;
            assert_eq!((*queue).list.iter().map(|item| item.get()).collect::<Vec<_>>(), vec![first, inserted, second]);
            while let Some(item) = (*queue).list.pop_front() { cmdq_remove(item); }
            crate::src::reactor::event_loop();
            assert_eq!(Rc::strong_count(&retained), 1);
            drop(retained);
            crate::src::reactor::event_loop();
            assert!(observer.upgrade().is_none());
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn execution_context_and_target_do_not_replace_queue_ownership() {
        unsafe {
            let owner = client::new();
            let context = client::new();
            let target = client::new();
            let observer = Rc::downgrade(&owner);
            let mut queue = cmdq_new();
            let item_owner = cmdq_get_callback_owned(c"accessor test".as_ptr(), None);
            let item = &mut *item_owner.get();
            item.queue = &mut *queue;
            item.client_owner = Some(owner);
            item.client = Rc::downgrade(&context);
            item.target_client = Rc::downgrade(&target);
            assert!(Rc::ptr_eq(&cmdq_get_client(Some(&item)).unwrap(), &context));
            assert!(Rc::ptr_eq(&cmdq_get_target_client(Some(&item)).unwrap(), &target));
            drop(context);
            drop(target);
            assert!(cmdq_get_client(Some(&item)).is_none());
            assert!(cmdq_get_target_client(Some(&item)).is_none());
            assert!(observer.upgrade().is_some());
            cmdq_remove(item_owner);
            crate::src::reactor::event_loop();
            assert!(observer.upgrade().is_none());
            crate::src::reactor::shutdown_runtime();
        }
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
    fn client_owns_queue_while_cancelled_items_release_their_payloads() {
        let before = DROPPED.load(Ordering::SeqCst);
        unsafe {
            let mut queue = cmdq_new();
            let address = &mut *queue as *mut cmdq_list;
            let item_allocation = cmdq_get_callback_owned(c"queued-cancel".as_ptr(), None);
            let item = item_allocation.get();
            let payload = Payload;
            cmdq_set_cancel_callback(&mut *item, Box::new(move || drop(payload)));
            (*item).queue = address;
            // Detached item transfer remains the next queue ownership boundary.
            queue.list.push_back(item_allocation);
            let owner = client::new();
            let client = &mut *owner.get();
            client.queue = Some(queue);
            assert_eq!(cmdq_get(Some(&owner)), address);
            cmdq_remove((*address).list.pop_front().unwrap());
            assert_eq!(DROPPED.load(Ordering::SeqCst), before + 1);
            assert!(client.queue.as_ref().unwrap().list.is_empty());
            drop(client.queue.take());
            drop(owner);
        }
    }

}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn removal_rejects_an_extra_strong_owner_before_cleanup() {
        unsafe {
            let mut queue = cmdq_new();
            let item = cmdq_get_callback_owned(c"extra owner".as_ptr(), None);
            (*item.get()).queue = &mut *queue;
            let observer = Rc::downgrade(&item);
            let guard = item.clone();
            let cancelled = Rc::new(Cell::new(false));
            let called = cancelled.clone();
            cmdq_set_cancel_callback(&mut *item.get(), Box::new(move || called.set(true)));
            queue.list.push_back(item);
            let owner = queue.list.pop_front().unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cmdq_remove(owner)));
            assert!(result.is_err());
            assert!(!cancelled.get());
            assert!(!(*guard.get()).removed);
            cmdq_remove(guard);
            assert!(cancelled.get());
            assert!(observer.upgrade().is_none());
        }
    }

    #[test]
    fn dropping_without_removal_panics_and_does_not_run_cancellation() {
        unsafe {
            let item = cmdq_get_callback_owned(c"unreleased detached item".as_ptr(), None);
            let observer = Rc::downgrade(&item);
            let cancelled = Rc::new(Cell::new(false));
            let called = cancelled.clone();
            cmdq_set_cancel_callback(&mut *item.get(), Box::new(move || called.set(true)));
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(item)));
            let panic = result.expect_err("dropping without removal must panic");
            assert_eq!(panic.downcast_ref::<String>().map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied()),
                Some("command item dropped without cmdq_remove"));
            assert!(observer.upgrade().is_none());
            assert!(!cancelled.get());
        }
    }

    #[test]
    fn group_removal_cancels_only_matching_suffix_items() {
        unsafe {
            let client = client::new();
            (*client.get()).queue = Some(cmdq_new());
            let cancelled = Rc::new(Cell::new(0));
            let first = cmdq_get_callback_owned(c"first".as_ptr(), None);
            let same = cmdq_get_callback_owned(c"same group".as_ptr(), None);
            let other = cmdq_get_callback_owned(c"other group".as_ptr(), None);
            (*first.get()).group = 7;
            (*same.get()).group = 7;
            (*other.get()).group = 9;
            let calls = cancelled.clone();
            cmdq_set_cancel_callback(&mut *same.get(), Box::new(move || calls.set(calls.get() + 1)));
            (*same.get()).next = Some(other.clone());
            let same_observer = Rc::downgrade(&same);
            (*first.get()).next = Some(same);
            cmdq_append(Some(&client), first.clone());
            cmdq_remove_group(&first);
            assert_eq!(cancelled.get(), 1);
            let queue = (*client.get()).queue.as_ref().unwrap();
            assert_eq!(queue.list.len(), 2);
            assert!(Rc::ptr_eq(queue.list.front().unwrap(), &first));
            assert!(Rc::ptr_eq(queue.list.back().unwrap(), &other));
            assert!(same_observer.upgrade().is_none());
            drop((first, other));
            let queue = cmdq_get(Some(&client));
            while let Some(item) = (*queue).list.pop_front() { cmdq_remove(item); }
            drop(client);
            crate::src::reactor::event_loop();
            crate::src::reactor::shutdown_runtime();
        }
    }
}

use crate::src::arguments::args_string;
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_session, event_payload_set_string,
    event_payload_set_target,
};
use crate::src::ffi::libc::strcmp;
use crate::src::format::format_single_from_target_cstring;
use crate::src::server_fn::server_status_session;
use crate::src::session::sessions;
use crate::src::session::{session_find, session_replace_name, sessions_insert, sessions_remove};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::events::event_payload;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::tmux::{check_name, clean_name_cstring};
use std::ffi::CStr;

#[no_mangle]
pub static mut cmd_rename_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"rename-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"rename\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-t target-session] new-name\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_rename_session_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_rename_session_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
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
    let tmp = format_single_from_target_cstring(item, args_string(args, 0 as u_int));
    if check_name(tmp.as_ptr()) == 0 {
        cmdq_error(
            item,
            b"invalid session name: %s\0" as *const u8 as *const ::core::ffi::c_char,
            tmp.as_ptr(),
        );
        return CMD_RETURN_ERROR;
    }
    let newname = clean_name_cstring(CStr::from_ptr(tmp.as_ptr()), 0)
        .expect("check_name validated the session name");
    if strcmp(newname.as_ptr(), ((*s).name).as_ptr().cast_mut()) == 0 as ::core::ffi::c_int {
        return CMD_RETURN_NORMAL;
    }
    if !session_find(newname.as_ptr()).is_null() {
        cmdq_error(
            item,
            b"duplicate session: %s\0" as *const u8 as *const ::core::ffi::c_char,
            newname.as_ptr(),
        );
        return CMD_RETURN_ERROR;
    }
    ep = event_payload_create();
    cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_string(
        ep,
        b"old_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
    );
    event_payload_set_string(
        ep,
        b"new_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        newname.as_ptr(),
    );
    sessions_remove(&raw mut sessions, s);
    drop(session_replace_name(&mut *s, newname));
    sessions_insert(&raw mut sessions, s);
    server_status_session(s);
    events_fire(
        b"session-renamed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
    return CMD_RETURN_NORMAL;
}

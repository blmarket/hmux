//! Authoritative command objects, queues, parsing records, and scalar domains.

use super::abi::{time_t, u_int};
use super::account::group;
use super::arguments::{args, args_parse};
use super::client::client;
use super::event::event;
use super::format::format_tree;
use super::key::key_event;
use super::pane::window_pane;
use super::session::session;
use super::window::{window, winlink};
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;

pub type cmd_find_type = ::core::ffi::c_uint;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;

pub type cmd_parse_status = ::core::ffi::c_uint;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;

pub const CMD_PARSE_QUIET: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_PARSE_PARSEONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_STARTSERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_LIST_PRINT_ESCAPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_LIST_PRINT_NO_GROUPS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_PREFER_UNATTACHED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMDQ_STATE_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_FIND_WINDOW_INDEX: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_TARGET_PANE_USAGE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"[-t target-pane]\0")
};
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMD_TARGET_CLIENT_USAGE: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"[-t target-client]\0")
};
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_FIND_QUIET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_SESSION: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_WINDOW: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_TARGET_SESSION_USAGE: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"[-t target-session]\0")
};
pub const CMD_PARSE_NOALIAS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_PARSE_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_PARSE_ONEGROUP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_PARSE_MAX_ENVIRON_LEN: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const CMDQ_STATE_CONTROL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMDQ_STATE_NOHOOKS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMDQ_FIRED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMDQ_WAITING: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_BUFFER_USAGE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"[-b buffer-name]\0")
};
pub const CMD_TARGET_WINDOW_USAGE: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"[-t target-window]\0")
};
pub const CMD_SOURCE_FILE_DEPTH_LIMIT: ::core::ffi::c_int = 50 as ::core::ffi::c_int;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn command_domains_match_translated_c_baseline() {
        assert_eq!(size_of::<cmd_retval>(), 4);
        assert_eq!(align_of::<cmd_retval>(), 4);
        assert_eq!(size_of::<cmd_find_type>(), 4);
        assert_eq!(align_of::<cmd_find_type>(), 4);
        assert_eq!(CMD_RETURN_ERROR, -1);
        assert_eq!(CMD_RETURN_NORMAL, 0);
        assert_eq!(CMD_RETURN_WAIT, 1);
        assert_eq!(CMD_RETURN_STOP, 2);
        assert_eq!(CMD_FIND_PANE, 0);
        assert_eq!(CMD_FIND_WINDOW, 1);
        assert_eq!(CMD_FIND_SESSION, 2);
        assert_eq!(size_of::<cmd_parse_status>(), 4);
        assert_eq!(align_of::<cmd_parse_status>(), 4);
        assert_eq!(CMD_PARSE_ERROR, 0);
        assert_eq!(CMD_PARSE_SUCCESS, 1);
    }
}

#[repr(C)]
/// Box-owned by its command queue after insertion. Command and callback
/// constructors return a stable address with a borrowed name;
/// `cmdq_remove` unlinks and drops it.
pub struct cmdq_item {
    pub name: *mut ::core::ffi::c_char,
    pub queue: *mut cmdq_list,
    pub next: *mut cmdq_item,
    pub client: *mut client,
    pub target_client: *mut client,
    pub type_0: cmdq_type,
    pub group: u_int,
    pub number: u_int,
    pub time: time_t,
    pub flags: ::core::ffi::c_int,
    pub state: *mut cmdq_state,
    pub source: cmd_find_state,
    pub target: cmd_find_state,
    pub cmdlist: *mut cmd_list,
    pub cmd: *mut cmd,
    pub cb: cmdq_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: cmdq_item_entry,
}

pub type cmds = Vec<*mut cmd>;

#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned by a client until `cmdq_free`; the lazy global queue lives for
/// the process. An empty queue's tail link points into this stable record.
pub struct cmdq_list {
    pub item: *mut cmdq_item,
    pub list: cmdq_item_list,
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

#[repr(C)]
/// Box-owned from `cmd_list_new` until the last explicit reference is freed.
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    /// Box-owned command pointer array, released with the final `cmd_list_free` reference.
    pub list: *mut Vec<*mut cmd>,
}

#[repr(C)]
/// Box-owned from `cmd_parse` or `cmd_copy` through `cmd_free`.
pub struct cmd {
    pub entry: *const cmd_entry,
    pub args: *mut args,
    pub group: u_int,
    pub file: *mut ::core::ffi::c_char,
    pub line: u_int,
    pub parse_flags: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: *const ::core::ffi::c_char,
    pub alias: *const ::core::ffi::c_char,
    pub args: args_parse,
    pub usage: *const ::core::ffi::c_char,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}

#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned while its explicit `references` count is nonzero; queue items
/// and callers hold counted raw pointers released through `cmdq_free_state`.
pub struct cmdq_state {
    pub references: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub formats: *mut format_tree,
    pub event: key_event,
    pub current: cmd_find_state,
}

pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdq_item_entry {
    pub tqe_next: *mut cmdq_item,
    pub tqe_prev: *mut *mut cmdq_item,
}

pub type cmdq_type = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdq_item_list {
    pub tqh_first: *mut cmdq_item,
    pub tqh_last: *mut *mut cmdq_item,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct wait_item {
    pub item: *mut cmdq_item,
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: *mut cmd_list,
    pub error: *mut ::core::ffi::c_char,
}

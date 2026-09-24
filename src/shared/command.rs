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
use std::ffi::{CStr, CString};
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;

pub type cmd_find_type = ::core::ffi::c_uint;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;

/// Owns command argument strings. The pointer cache exists only as a scoped
/// view for translated C APIs and is rebuilt whenever the collection changes.
/// The strings themselves remain byte preserving and are freed by Rust.
#[derive(Default)]
pub struct OwnedArgv {
    strings: Vec<Option<CString>>,
    pointers: Vec<*mut ::core::ffi::c_char>,
}

impl OwnedArgv {
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn push_cstr(&mut self, value: &CStr) {
        self.strings.push(Some(value.to_owned()));
        self.rebuild_pointer_view();
    }

    pub fn push_optional_cstr(&mut self, value: Option<&CStr>) {
        self.strings.push(value.map(CStr::to_owned));
        self.rebuild_pointer_view();
    }

    pub fn prepend_cstr(&mut self, value: &CStr) {
        self.strings.insert(0, Some(value.to_owned()));
        self.rebuild_pointer_view();
    }

    pub fn copy(&self) -> Self {
        let mut result = Self {
            strings: self.strings.clone(),
            pointers: Vec::new(),
        };
        result.rebuild_pointer_view();
        result
    }

    /// Copy the first `argc` C strings. Null entries are retained as nulls,
    /// matching `cmd_copy_argv`'s historical behavior.
    pub unsafe fn copy_from_raw(
        argc: ::core::ffi::c_int,
        argv: *mut *mut ::core::ffi::c_char,
    ) -> Self {
        let mut result = Self::default();
        for index in 0..argc.max(0) as usize {
            let value = *argv.add(index);
            if value.is_null() {
                result.push_optional_cstr(None);
            } else {
                result.push_optional_cstr(Some(CStr::from_ptr(value)));
            }
        }
        result
    }

    /// Decode the length-delimited argv wire representation. As in the C
    /// implementation, the final byte is forced to NUL before token scans.
    pub fn unpack(buffer: &mut [u8], argc: ::core::ffi::c_int) -> Result<Self, ()> {
        if argc == 0 {
            return Ok(Self::default());
        }
        if argc < 0 || argc > 1000 || buffer.is_empty() {
            return Err(());
        }
        *buffer.last_mut().expect("nonempty buffer") = 0;
        let mut result = Self::default();
        let mut start = 0;
        for _ in 0..argc {
            if start >= buffer.len() {
                return Err(());
            }
            let tail = &buffer[start..];
            let end = tail.iter().position(|byte| *byte == 0).ok_or(())?;
            result.push_cstr(CStr::from_bytes_with_nul(&tail[..=end]).map_err(|_| ())?);
            start += end + 1;
        }
        Ok(result)
    }

    /// Borrow a null-terminated pointer array for synchronous C calls. The
    /// returned pointer is invalidated by the next mutation of this owner.
    pub fn as_mut_ptr(&mut self) -> *mut *mut ::core::ffi::c_char {
        if self.strings.is_empty() {
            ::core::ptr::null_mut()
        } else {
            self.pointers.as_mut_ptr()
        }
    }

    pub fn argc(&self) -> ::core::ffi::c_int {
        ::core::ffi::c_int::try_from(self.strings.len()).expect("argv length exceeds c_int")
    }

    fn rebuild_pointer_view(&mut self) {
        self.pointers.clear();
        self.pointers.reserve(self.strings.len() + 1);
        self.pointers.extend(self.strings.iter_mut().map(|string| {
            string
                .as_mut()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
        }));
        self.pointers.push(::core::ptr::null_mut());
    }
}

#[cfg(test)]
mod owned_argv_tests {
    use super::OwnedArgv;
    use std::ffi::CStr;

    #[test]
    fn unpack_preserves_empty_and_non_utf8_arguments_and_terminates_view() {
        let mut bytes = [b'a', 0, 0, 0xff, 0, b'x'];
        let mut argv = OwnedArgv::unpack(&mut bytes, 3).expect("three arguments fit");
        assert_eq!(bytes[5], 0);
        assert_eq!(argv.argc(), 3);
        let pointers = argv.as_mut_ptr();
        assert_eq!(unsafe { CStr::from_ptr(*pointers.add(0)) }.to_bytes(), b"a");
        assert_eq!(unsafe { CStr::from_ptr(*pointers.add(1)) }.to_bytes(), b"");
        assert_eq!(unsafe { CStr::from_ptr(*pointers.add(2)) }.to_bytes(), &[0xff]);
        assert!(unsafe { (*pointers.add(3)).is_null() });

        let mut empty = OwnedArgv::unpack(&mut [], 0).expect("empty argv is valid");
        assert!(empty.as_mut_ptr().is_null());
    }

    #[test]
    fn unpack_rejects_more_arguments_than_the_wire_buffer_contains() {
        let mut bytes = [b'x', 0];
        assert!(OwnedArgv::unpack(&mut bytes, 2).is_err());
    }
}

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
pub struct cmdq_item {
    pub name: Option<std::ffi::CString>,
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
    pub(crate) error: Option<std::ffi::CString>,
    pub(crate) cancel_data: Option<unsafe fn(*mut ::core::ffi::c_void)>,
    pub(crate) wait_file: *mut super::client::client_file,
}

impl cmdq_item {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            queue: unsafe { ::core::mem::zeroed() },
            next: unsafe { ::core::mem::zeroed() },
            client: unsafe { ::core::mem::zeroed() },
            target_client: unsafe { ::core::mem::zeroed() },
            type_0: unsafe { ::core::mem::zeroed() },
            group: unsafe { ::core::mem::zeroed() },
            number: unsafe { ::core::mem::zeroed() },
            time: unsafe { ::core::mem::zeroed() },
            flags: unsafe { ::core::mem::zeroed() },
            state: unsafe { ::core::mem::zeroed() },
            source: unsafe { ::core::mem::zeroed() },
            target: unsafe { ::core::mem::zeroed() },
            cmdlist: unsafe { ::core::mem::zeroed() },
            cmd: unsafe { ::core::mem::zeroed() },
            cb: unsafe { ::core::mem::zeroed() },
            data: unsafe { ::core::mem::zeroed() },
            entry: unsafe { ::core::mem::zeroed() },
            error: Default::default(),
            cancel_data: Default::default(),
            wait_file: Default::default(),
        }
    }
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
pub struct cmd {
    pub entry: *const cmd_entry,
    pub args: *mut args,
    pub group: u_int,
    pub file: Option<std::ffi::CString>,
    pub line: u_int,
    pub parse_flags: ::core::ffi::c_int,
}

impl cmd {
    pub fn empty() -> Self {
        Self {
            entry: unsafe { ::core::mem::zeroed() },
            args: unsafe { ::core::mem::zeroed() },
            group: unsafe { ::core::mem::zeroed() },
            file: Default::default(),
            line: unsafe { ::core::mem::zeroed() },
            parse_flags: unsafe { ::core::mem::zeroed() },
        }
    }
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

pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: *mut cmd_list,
    pub error: Option<CString>,
}

impl cmd_parse_result {
    pub fn empty() -> Self {
        Self {
            status: CMD_PARSE_ERROR,
            cmdlist: ::core::ptr::null_mut(),
            error: None,
        }
    }
}

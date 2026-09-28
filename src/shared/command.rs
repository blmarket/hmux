//! Authoritative command objects, queues, parsing records, and scalar domains.

use super::abi::{time_t, u_int};
use super::arguments::{args, args_parse};
use super::client::client;
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

/// Decode the length-delimited argv wire representation. The final byte is
/// forced to NUL before token scans, matching the old C decoder.
pub fn unpack_argv(buffer: &mut [u8], argc: ::core::ffi::c_int) -> Result<Vec<CString>, ()> {
    if argc == 0 {
        return Ok(Vec::new());
    }
    if argc < 0 || argc > 1000 || buffer.is_empty() {
        return Err(());
    }
    *buffer.last_mut().expect("nonempty buffer") = 0;
    let mut result = Vec::with_capacity(argc as usize);
    let mut start = 0;
    for _ in 0..argc {
        if start >= buffer.len() {
            return Err(());
        }
        let tail = &buffer[start..];
        let end = tail.iter().position(|byte| *byte == 0).ok_or(())?;
        result.push(
            CStr::from_bytes_with_nul(&tail[..=end])
                .map_err(|_| ())?
                .to_owned(),
        );
        start += end + 1;
    }
    Ok(result)
}

#[cfg(test)]
mod owned_argv_tests {
    use super::unpack_argv;

    #[test]
    fn unpack_preserves_empty_and_non_utf8_arguments_and_terminates_view() {
        let mut bytes = [b'a', 0, 0, 0xff, 0, b'x'];
        let argv = unpack_argv(&mut bytes, 3).expect("three arguments fit");
        assert_eq!(bytes[5], 0);
        assert_eq!(argv.len(), 3);
        assert_eq!(argv[0].as_bytes(), b"a");
        assert_eq!(argv[1].as_bytes(), b"");
        assert_eq!(argv[2].as_bytes(), &[0xff]);
        assert!(unpack_argv(&mut [], 0)
            .expect("empty argv is valid")
            .is_empty());
    }

    #[test]
    fn unpack_rejects_more_arguments_than_the_wire_buffer_contains() {
        let mut bytes = [b'x', 0];
        assert!(unpack_argv(&mut bytes, 2).is_err());
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
pub const CMD_TARGET_PANE_USAGE: &'static std::ffi::CStr = c"[-t target-pane]";
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMD_TARGET_CLIENT_USAGE: &'static std::ffi::CStr = c"[-t target-client]";
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_FIND_QUIET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_SESSION: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_WINDOW: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_TARGET_SESSION_USAGE: &'static std::ffi::CStr = c"[-t target-session]";
pub const CMD_PARSE_NOALIAS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_PARSE_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_PARSE_ONEGROUP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_PARSE_MAX_ENVIRON_LEN: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const CMDQ_STATE_CONTROL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMDQ_STATE_NOHOOKS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMDQ_FIRED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMDQ_WAITING: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_BUFFER_USAGE: &'static std::ffi::CStr = c"[-b buffer-name]";
pub const CMD_TARGET_WINDOW_USAGE: &'static std::ffi::CStr = c"[-t target-window]";
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

pub struct cmdq_item {
    /// Observe this allocation across detached and queued ownership transfer.
    pub(crate) observer: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
    pub name: Option<std::ffi::CString>,
    pub queue: *mut cmdq_list,
    /// Detached-item chain link; each linked item has its own sole owner.
    pub next: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
    /// Nonowning execution context, which can temporarily differ from the queue owner.
    pub client: std::rc::Weak<std::cell::UnsafeCell<client>>,
    pub client_owner: Option<std::rc::Rc<std::cell::UnsafeCell<client>>>,
    /// Nonowning command target; upgrade before accessing the client.
    pub target_client: std::rc::Weak<std::cell::UnsafeCell<client>>,
    pub type_0: cmdq_type,
    pub group: u_int,
    pub number: u_int,
    pub time: time_t,
    pub flags: ::core::ffi::c_int,
    pub state: Option<std::rc::Rc<cmdq_state>>,
    pub source: cmd_find_state,
    pub target: cmd_find_state,
    pub cmdlist: Option<std::rc::Rc<std::cell::RefCell<cmd_list>>>,
    pub cmd: refbox::Weak<cmd>,
    pub cb: cmdq_cb,
    pub(crate) cancel_data: Option<Box<dyn FnOnce()>>,
    pub(crate) wait_file: std::rc::Weak<std::cell::UnsafeCell<super::client::client_file>>,
}

impl cmdq_item {
    /// Compatibility view while a detached item still owns itself.
    pub fn next_ptr(&self) -> *mut cmdq_item {
        if self.next.strong_count() == 0 {
            std::ptr::null_mut()
        } else {
            self.next.as_ptr().cast_mut().cast()
        }
    }

    /// Project a legacy command pointer while its list owner remains live.
    pub fn cmd_ptr(&self) -> *mut cmd {
        if self.cmd.is_alive() {
            self.cmd.as_ptr().cast_mut()
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn empty() -> Self {
        Self {
            observer: Default::default(),
            name: Default::default(),
            queue: Default::default(),
            next: Default::default(),
            client: Default::default(),
            client_owner: None,
            target_client: Default::default(),
            type_0: Default::default(),
            group: Default::default(),
            number: Default::default(),
            time: Default::default(),
            flags: Default::default(),
            state: Default::default(),
            source: Default::default(),
            target: Default::default(),
            cmdlist: Default::default(),
            cmd: Default::default(),
            cb: Default::default(),
            cancel_data: Default::default(),
            wait_file: Default::default(),
        }
    }
}

#[repr(C)]
/// Box-owned by a client until client destruction; the lazy global queue lives for
/// the process. The deque owns stable command item allocations.
pub struct cmdq_list {
    /// Current execution position; the deque remains its sole owner.
    pub item: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
    pub list: std::collections::VecDeque<std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
}

impl cmdq_list {
    pub fn first_ptr(&self) -> *mut cmdq_item {
        self.list.front().map_or(std::ptr::null_mut(), |item| item.get())
    }

    pub fn position(&self, item: *mut cmdq_item) -> usize {
        self.list
            .iter()
            .position(|owner| std::ptr::eq(owner.get(), item))
            .expect("command item belongs to queue")
    }
}

/// Saved, nonowning target identities. An empty observer means unspecified;
/// an expired observer still identifies a target that has disappeared.
/// Clone/drop these handles normally: this record must never be zeroed or
/// bitwise-copied, including when embedded in a mode, prompt, or queue item.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_find_state {
    pub flags: ::core::ffi::c_int,
    pub s: std::rc::Weak<std::cell::UnsafeCell<session>>,
    pub wl: refbox::Weak<winlink>,
    pub w: std::rc::Weak<std::cell::UnsafeCell<window>>,
    pub wp: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub idx: ::core::ffi::c_int,
}

impl cmd_find_state {
    /// Borrow a live target address for the translated, single-threaded callers.
    ///
    /// # Safety
    /// The caller must keep the model alive throughout use of the returned pointer
    /// and must re-resolve targets after callbacks that can destroy them.
    pub unsafe fn s_ptr(&self) -> *mut session {
        if self.s.strong_count() == 0 {
            std::ptr::null_mut()
        } else {
            self.s.as_ptr().cast_mut().cast()
        }
    }

    /// # Safety
    /// A nonnull pointer must refer to a live, shared model allocation.
    pub unsafe fn set_s(&mut self, ptr: *mut session) {
        self.s = ptr
            .as_ref()
            .map_or_else(std::rc::Weak::new, |value| value.observer.clone());
    }

    /// Borrow a live target address for the translated, single-threaded callers.
    ///
    /// # Safety
    /// The caller must keep the model alive throughout use of the returned pointer
    /// and must re-resolve targets after callbacks that can destroy them.
    pub unsafe fn w_ptr(&self) -> *mut window {
        if self.w.strong_count() == 0 {
            std::ptr::null_mut()
        } else {
            self.w.as_ptr().cast_mut().cast()
        }
    }

    /// # Safety
    /// A nonnull pointer must refer to a live, shared model allocation.
    pub unsafe fn set_w(&mut self, ptr: *mut window) {
        self.w = ptr
            .as_ref()
            .map_or_else(std::rc::Weak::new, |value| value.observer.clone());
    }

    /// Borrow a live target address for the translated, single-threaded callers.
    ///
    /// # Safety
    /// The caller must keep the model alive throughout use of the returned pointer
    /// and must re-resolve targets after callbacks that can destroy them.
    pub unsafe fn wp_ptr(&self) -> *mut window_pane {
        if self.wp.strong_count() == 0 {
            std::ptr::null_mut()
        } else {
            self.wp.as_ptr().cast_mut().cast()
        }
    }

    /// # Safety
    /// A nonnull pointer must refer to a live, shared model allocation.
    pub unsafe fn set_wp(&mut self, ptr: *mut window_pane) {
        self.wp = ptr
            .as_ref()
            .map_or_else(std::rc::Weak::new, |value| value.observer.clone());
    }

    /// # Safety
    /// The caller must keep the winlink alive and uphold its borrowing rules
    /// throughout pointer use. Re-resolve after callbacks that can remove it.
    pub unsafe fn wl_ptr(&self) -> *mut winlink {
        if self.wl.is_alive() {
            self.wl.as_ptr().cast_mut()
        } else {
            std::ptr::null_mut()
        }
    }

    /// # Safety
    /// A nonnull pointer must refer to a live RefBox-owned winlink.
    pub unsafe fn set_wl(&mut self, ptr: *mut winlink) {
        self.wl = ptr
            .as_ref()
            .map_or_else(refbox::Weak::new, |value| value.observer.clone());
    }
}

#[repr(C)]
/// Owned by ordinary Rc handles from construction through final drop.
#[derive(Default)]
pub struct cmd_list {
    pub group: u_int,
    /// Sole command owners, released with the final command-list reference.
    pub list: Vec<refbox::RefBox<cmd>>,
}

#[repr(C)]
pub struct cmd {
    pub entry: &'static cmd_entry,
    pub args: Option<Box<args>>,
    pub group: u_int,
    pub file: Option<std::ffi::CString>,
    pub line: u_int,
    pub parse_flags: ::core::ffi::c_int,
}

impl cmd {
    pub fn new(entry: &'static cmd_entry) -> Self {
        Self {
            entry,
            args: Default::default(),
            group: Default::default(),
            file: Default::default(),
            line: Default::default(),
            parse_flags: Default::default(),
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
    pub name: &'static std::ffi::CStr,
    pub alias: Option<&'static std::ffi::CStr>,
    pub args: args_parse,
    pub usage: &'static std::ffi::CStr,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}

#[repr(C)]
/// Queue items and callers share ownership through ordinary Rc handles.
pub struct cmdq_state {
    pub flags: ::core::ffi::c_int,
    pub formats: std::cell::RefCell<Option<super::format::FormatTreeOwner>>,
    pub event: key_event,
    pub current: std::cell::RefCell<cmd_find_state>,
}

impl cmdq_state {
    /// Copy the weak target handles and release the borrow before callbacks run.
    pub fn current_snapshot(&self) -> cmd_find_state {
        self.current.borrow().clone()
    }
}

pub type cmdq_cb = Option<Box<dyn FnOnce(std::ptr::NonNull<cmdq_item>) -> cmd_retval>>;

pub type cmdq_type = ::core::ffi::c_uint;

#[derive(Clone, Default)]
#[repr(C)]
pub struct wait_item {
    pub item: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_parse_input {
    pub flags: ::core::ffi::c_int,
    pub file: Option<::std::ffi::CString>,
    pub line: u_int,
    pub item: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
    /// Parser context observes the client; prepared commands own it separately.
    pub c: std::rc::Weak<std::cell::UnsafeCell<client>>,
    pub fs: cmd_find_state,
}

impl cmd_parse_input {
    pub unsafe fn set_item(&mut self, item: *mut cmdq_item) {
        self.item = item.as_ref().map_or_else(std::rc::Weak::new, |item| item.observer.clone());
    }

    pub fn file_ptr(&self) -> *const ::core::ffi::c_char {
        self.file
            .as_ref()
            .map_or(::core::ptr::null(), |file| file.as_ptr())
    }
}

pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: Option<std::rc::Rc<std::cell::RefCell<cmd_list>>>,
    pub error: Option<CString>,
}

impl cmd_parse_result {
    pub fn take_cmdlist(&mut self) -> Option<std::rc::Rc<std::cell::RefCell<cmd_list>>> {
        self.cmdlist.take()
    }

    pub fn empty() -> Self {
        Self {
            status: CMD_PARSE_ERROR,
            cmdlist: None,
            error: None,
        }
    }
}

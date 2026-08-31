//! Type declarations shared by every generated module.
//!
//! `c2rust` emitted these once per translation unit, which made each
//! module's `grid_cell` a distinct Rust type.  They are collected here
//! so that a name means one type crate-wide; every module glob-imports
//! this module, and the few modules that hold a private definition of a
//! type that is opaque here keep their own copy, which shadows the glob.

use ::core::cell::UnsafeCell;
use ::std::fmt;
use ::std::ops::{Deref, DerefMut};
use ::std::rc::{Rc, Weak};

use crate::screen::screen_free;

unsafe extern "C" {
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type _IO_wide_data;
    pub type dirent;
    pub type re_dfa_t;
    pub type sockaddr_at;
    pub type sockaddr_ax25;
    pub type sockaddr_dl;
    pub type sockaddr_eon;
    pub type sockaddr_in6;
    pub type sockaddr_inarp;
    pub type sockaddr_ipx;
    pub type sockaddr_iso;
    pub type sockaddr_ns;
    pub type sockaddr_x25;
    pub type term;
}

pub use crate::reactor::{Buf, IoHandle, SignalHandle, Stream, TimerHandle};
pub type TERMINAL = term;
pub use crate::arguments::args;
pub use crate::arguments::args_command_state;
pub use crate::cmd::cmd;
pub use crate::cmd::cmd_command_prompt_cdata;
pub use crate::cmd::cmd_confirm_before_data;
pub use crate::cmd::cmd_display_panes_data;
pub use crate::cmd::cmd_if_shell_data;
pub use crate::cmd::cmd_load_buffer_data;
pub use crate::cmd::cmd_run_shell_data;
pub use crate::cmd::cmd_source_file_data;
pub use crate::cmd::cmdq_item;
pub use crate::cmd::cmdq_list;
pub use crate::cmd::cmds;
pub use crate::compat::ibufqueue;
pub use crate::compat::msgbuf;
pub use crate::compat::msghdr;
pub use crate::control::control_state;
pub use crate::environ::environ_entry;
use crate::environ::environ_t;
pub use crate::format::format_job;
pub use crate::format::format_job_tree;
pub use crate::format::format_tree;
use crate::grid::HyperlinksRef;
pub use crate::grid::hyperlinks;
pub use crate::input::input_request;
pub use crate::input::{InputCtxRef, input_ctx};
pub use crate::job::job;
pub use crate::key_bindings::{key_binding, key_bindings, key_table};
pub use crate::modes::mode_tree_data;
pub use crate::modes::mode_tree_item;
pub use crate::modes::mode_tree_menu;
use crate::modes::window_buffer_editdata;
use crate::modes::window_buffer_modedata;
use crate::modes::window_clock_mode_data;
use crate::modes::window_copy_mode_data;
use crate::modes::{window_client_itemdata, window_client_modedata};
pub use crate::modes::{window_customize_itemdata, window_customize_modedata};
pub use crate::modes::{window_tree_itemdata, window_tree_modedata};
use crate::notify::notify_entry;
pub use crate::options::options;
pub use crate::options::options_array_item_t;
pub use crate::options::options_entry;
pub use crate::overlay::menu_data;
pub use crate::overlay::{PopupDataRef, PopupDataWeak, popup_data};
pub use crate::paste::paste_buffer;
pub use crate::proc::tmuxpeer;
pub use crate::proc::tmuxproc;
pub use crate::screen::screen_sel;
pub use crate::screen::screen_titles;
pub use crate::screen::screen_write_citem;
pub use crate::screen::screen_write_cline;
pub use crate::server::server_acl_user;
pub use crate::session::session;
pub use crate::session::{session_group, session_groups_t};
pub use crate::status::status_prompt_menu;
use crate::terminfo::TtyCode;
pub use crate::tty::tty_key;
pub use crate::window::window_pane_input_data;
pub use ::libc::{sockaddr_storage, termios, utsname};
pub type __off64_t = ::core::ffi::c_long;
pub type __off_t = ::core::ffi::c_long;
pub type __uint64_t = u64;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    #[bitfield(name = "_flags2", ty = "::core::ffi::c_int", bits = "0..=23")]
    pub _flags2: [u8; 3],
    pub _short_backupbuf: [::core::ffi::c_char; 1],
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused3: ::core::ffi::c_int,
    pub _total_written: __uint64_t,
    pub _unused2: [::core::ffi::c_char; 8],
}
pub type FILE = _IO_FILE;
pub type _IO_lock_t = ();
pub type __blkcnt_t = ::core::ffi::c_long;
pub type __blksize_t = ::core::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type __builtin_va_list = [__va_list_tag; 1];
pub type __clock_t = ::core::ffi::c_long;
pub type __clockid_t = ::core::ffi::c_int;
pub type __dev_t = ::core::ffi::c_ulong;
pub type __gid_t = ::core::ffi::c_uint;
pub type __gnuc_va_list = __builtin_va_list;
pub type __ino_t = ::core::ffi::c_ulong;
pub type __int32_t = i32;
pub type __mode_t = ::core::ffi::c_uint;
pub type __nlink_t = ::core::ffi::c_ulong;
pub type __pid_t = ::core::ffi::c_int;
pub type __re_long_size_t = ::core::ffi::c_ulong;
pub type __sighandler_t = Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [::core::ffi::c_ulong; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union sigval {
    pub sival_int: ::core::ffi::c_int,
    pub sival_ptr: *mut ::core::ffi::c_void,
}
pub type __sigval_t = sigval;
pub type __size_t = usize;
pub type __socket_type = ::core::ffi::c_uint;
pub type __socklen_t = ::core::ffi::c_uint;
pub type __suseconds_t = ::core::ffi::c_long;
pub type __syscall_slong_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_int = ::core::ffi::c_uint;
pub type __u_short = ::core::ffi::c_ushort;
pub type __uid_t = ::core::ffi::c_uint;
pub type __uint16_t = u16;
pub type __uint32_t = u32;
pub type __uint8_t = u8;
pub type __useconds_t = ::core::ffi::c_uint;
pub type args_parse_type = ::core::ffi::c_uint;
pub type u_int = __u_int;
pub type args_parse_cb =
    Option<unsafe fn(&args, u_int, &mut Option<::std::ffi::CString>) -> args_parse_type>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse_t {
    pub template: &'static ::core::ffi::CStr,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
pub type bitstr_t = ::core::ffi::c_uchar;
pub type box_lines = ::core::ffi::c_int;
pub type cc_t = ::core::ffi::c_uchar;
pub type client_theme = ::core::ffi::c_uint;
pub type clockid_t = __clockid_t;
pub type cmd_find_type = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
pub type cmd_retval = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: &'static ::core::ffi::CStr,
    pub alias: Option<&'static ::core::ffi::CStr>,
    pub args: args_parse_t,
    pub usage: &'static ::core::ffi::CStr,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: unsafe fn(&cmd, *mut cmdq_item) -> cmd_retval,
}

/// A transpiled table that lives for the whole run and is only ever read.
///
/// The tables hold raw pointers to string literals, which are not [`Sync`], so
/// on their own they cannot be the shared `static`s they should be. Wrapping
/// one says once what is true of all of them. [`as_ptr`](Self::as_ptr) hands
/// out the `*mut` the transpiled walks expect, and [`Deref`] keeps the rest of
/// the call sites reading like the array underneath.
///
/// [`Deref`]: ::core::ops::Deref
#[repr(transparent)]
pub struct ReadOnly<T>(T);

unsafe impl<T> Sync for ReadOnly<T> {}

impl<T> ReadOnly<T> {
    pub const fn new(table: T) -> Self {
        ReadOnly(table)
    }
}

impl<T, const N: usize> ReadOnly<[T; N]> {
    /// The first element, for the walks that step to a null terminator.
    pub const fn as_ptr(&'static self) -> *mut T {
        self.0.as_ptr().cast_mut()
    }
}

impl<T> ::core::ops::Deref for ReadOnly<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[repr(C)]
pub struct cmd_list {
    pub group: u_int,
    pub list: Option<Box<cmds>>,
}

/// A strong owner of a parsed command list.
///
/// Command execution still uses raw pointers internally, but the pointer from
/// [`as_ptr`](Self::as_ptr) is only a borrowed view while this handle remains
/// alive. Command-list ownership must be transferred by cloning or dropping
/// this handle, never by copying the raw pointer.
#[derive(Clone)]
pub struct CmdListRef(Rc<UnsafeCell<cmd_list>>);

impl CmdListRef {
    pub(crate) fn new(value: cmd_list) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut cmd_list {
        self.0.get()
    }
}

/// The state a `source-file` run carries between the reads it starts. Several
/// client files may be reading for one run, so they share it and the last one
/// gone takes it with them.
#[derive(Clone)]
pub struct SourceFileRef(Rc<UnsafeCell<cmd_source_file_data>>);

impl SourceFileRef {
    pub(crate) fn new(value: cmd_source_file_data) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut cmd_source_file_data {
        self.0.get()
    }
}

impl fmt::Debug for SourceFileRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SourceFileRef")
            .field(&self.as_ptr())
            .finish()
    }
}

/// The state a pane's input read carries, shared the same way.
#[derive(Clone)]
pub struct PaneInputRef(Rc<UnsafeCell<window_pane_input_data>>);

impl PaneInputRef {
    pub(crate) fn new(value: window_pane_input_data) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut window_pane_input_data {
        self.0.get()
    }
}

impl fmt::Debug for PaneInputRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("PaneInputRef")
            .field(&self.as_ptr())
            .finish()
    }
}

impl PartialEq for CmdListRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CmdListRef {}

impl fmt::Debug for CmdListRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CmdListRef")
            .field(&self.as_ptr())
            .finish()
    }
}

/// A strong owner of a temporary status-line screen.
#[derive(Clone)]
pub(crate) struct StatusScreenRef(Rc<UnsafeCell<StatusScreenStorage>>);

pub(crate) struct StatusScreenStorage {
    pub(crate) value: screen,
}

impl StatusScreenRef {
    pub(crate) fn new(value: screen) -> Self {
        Self(Rc::new(UnsafeCell::new(StatusScreenStorage { value })))
    }

    pub(crate) fn as_ptr(&self) -> *mut screen {
        unsafe { &mut (*self.0.get()).value as *mut screen }
    }

    pub(crate) fn is_unique(&self) -> bool {
        Rc::strong_count(&self.0) == 1
    }

    /// Makes a non-owning observation of this screen.
    pub(crate) fn downgrade(&self) -> StatusScreenWeak {
        StatusScreenWeak(Rc::downgrade(&self.0))
    }
}

/// A non-owning observation of an overlay screen. The status line watches the
/// overlay it is drawing on this way, so that the message and prompt slots
/// alone decide how long the screen lives.
#[derive(Clone)]
pub(crate) struct StatusScreenWeak(Weak<UnsafeCell<StatusScreenStorage>>);

impl StatusScreenWeak {
    /// The screen, or null once the slots that held it have given it up.
    pub(crate) fn screen(&self) -> *mut screen {
        self.0
            .upgrade()
            .map_or(::core::ptr::null_mut(), |held| unsafe {
                &mut (*held.get()).value as *mut screen
            })
    }
}

impl Drop for StatusScreenStorage {
    fn drop(&mut self) {
        unsafe { screen_free(&raw mut self.value) };
    }
}

/// A strong owner of a window allocation. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is a borrowed compatibility view; an owning edge
/// must clone or drop this handle instead of retaining that pointer. Winlinks,
/// queued alerts and notifications use strong handles, while the id registry
/// and timer callbacks use [`WindowWeak`]. A pane's back-edge to its window is
/// neither: the window owns its panes by value, so the plain pointer cannot
/// outlive what it points at.
#[derive(Clone)]
pub(crate) struct WindowRef(Rc<UnsafeCell<WindowStorage>>);

#[derive(Clone)]
pub(crate) struct WindowWeak(Weak<UnsafeCell<WindowStorage>>);

pub(crate) struct WindowStorage {
    pub(crate) value: window,
    pub(crate) managed: bool,
}

impl WindowRef {
    pub(crate) fn new(value: window) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(WindowStorage {
            value,
            managed: true,
        })));
        crate::window::register_window_handle(&reference);
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut window {
        unsafe { &mut (*self.0.get()).value as *mut window }
    }

    pub(crate) fn downgrade(&self) -> WindowWeak {
        WindowWeak(Rc::downgrade(&self.0))
    }

    /// Gives up the teardown [`Drop`] would otherwise run. Only the tests take
    /// this, for the windows they hand-build outside the server's own trees.
    #[cfg(test)]
    pub(crate) fn mark_unmanaged(&self) {
        unsafe {
            (*self.0.get()).managed = false;
        }
    }
}

impl WindowWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowRef> {
        self.0.upgrade().map(WindowRef)
    }
}

/// A strong owner of a session allocation. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is a borrowed compatibility view; deferred jobs
/// and notifications own sessions by cloning this handle. The live-session
/// registry owns one handle while a session is discoverable, while clients
/// and target state remain observational back-pointers.
#[derive(Clone)]
pub(crate) struct SessionRef(Rc<UnsafeCell<SessionStorage>>);

#[derive(Clone)]
pub(crate) struct SessionWeak(Weak<UnsafeCell<SessionStorage>>);

pub(crate) struct SessionStorage {
    pub(crate) value: session,
}

impl SessionRef {
    pub(crate) fn new(value: session) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(SessionStorage { value })));
        crate::session::register_session_handle(&reference);
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut session {
        unsafe { &mut (*self.0.get()).value as *mut session }
    }

    pub(crate) fn downgrade(&self) -> SessionWeak {
        SessionWeak(Rc::downgrade(&self.0))
    }
}

impl SessionWeak {
    pub(crate) fn upgrade(&self) -> Option<SessionRef> {
        self.0.upgrade().map(SessionRef)
    }
}

/// A strong owner of a live client allocation. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is a compatibility view; non-owning code should
/// use [`with`](Self::with) for short immutable observations instead.
#[derive(Clone)]
pub(crate) struct ClientRef(Rc<UnsafeCell<ClientStorage>>);

#[derive(Clone)]
pub struct ClientWeak(Weak<UnsafeCell<ClientStorage>>);

pub(crate) struct ClientStorage {
    pub(crate) value: client,
}

impl ClientRef {
    pub(crate) fn new(value: client) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(ClientStorage { value })));
        crate::server::register_client_handle(&reference);
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut client {
        unsafe { &(*self.0.get()).value as *const client as *mut client }
    }

    pub(crate) fn with<R>(&self, f: impl FnOnce(&client) -> R) -> R {
        unsafe { f(&(*self.0.get()).value) }
    }

    pub(crate) fn downgrade(&self) -> ClientWeak {
        ClientWeak(Rc::downgrade(&self.0))
    }
}

impl ClientWeak {
    pub(crate) fn upgrade(&self) -> Option<ClientRef> {
        self.0.upgrade().map(ClientRef)
    }
}

impl Deref for ClientRef {
    type Target = client;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for ClientRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_ptr() }
    }
}

/// A strong owner of a live key-binding table. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is an observational view and is valid only while
/// this handle, or another strong handle to the same table, is alive. The
/// global table tree and each client key-table field own independent clones.
#[derive(Clone)]
pub(crate) struct KeyTableRef(Rc<UnsafeCell<key_table>>);

impl KeyTableRef {
    pub(crate) fn new(value: key_table) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut key_table {
        self.0.get()
    }
}

/// A strong owner of a client file operation. The raw pointer passed to I/O
/// callbacks is an observational view; deferred callbacks carry this handle
/// so completion cannot outlive the operation. The file's client pointer is
/// currently governed by the client's existing hold and is not an owning edge.
#[derive(Clone)]
pub(crate) struct ClientFileRef(Rc<UnsafeCell<client_file>>);

impl ClientFileRef {
    pub(crate) fn new(value: client_file) -> Self {
        Self(Rc::new(UnsafeCell::new(value)))
    }

    pub(crate) fn as_ptr(&self) -> *mut client_file {
        self.0.get()
    }
}

#[derive(Clone)]
pub struct ModeTreeDataRef(Rc<UnsafeCell<mode_tree_data>>);

#[derive(Clone)]
pub struct ModeTreeDataWeak(Weak<UnsafeCell<mode_tree_data>>);

impl ModeTreeDataRef {
    pub(crate) fn new(value: mode_tree_data) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe {
            (*reference.as_ptr()).owner = Some(reference.downgrade());
        }
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut mode_tree_data {
        self.0.get()
    }

    pub(crate) fn downgrade(&self) -> ModeTreeDataWeak {
        ModeTreeDataWeak(Rc::downgrade(&self.0))
    }
}

impl ModeTreeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<ModeTreeDataRef> {
        self.0.upgrade().map(ModeTreeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct WindowBufferModeDataRef(Rc<UnsafeCell<window_buffer_modedata>>);

#[derive(Clone)]
pub struct WindowBufferModeDataWeak(Weak<UnsafeCell<window_buffer_modedata>>);

impl WindowBufferModeDataRef {
    pub(crate) fn new(value: window_buffer_modedata) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe {
            (*reference.as_ptr()).owner = Some(reference.downgrade());
        }
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut window_buffer_modedata {
        self.0.get()
    }

    pub(crate) fn downgrade(&self) -> WindowBufferModeDataWeak {
        WindowBufferModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowBufferModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowBufferModeDataRef> {
        self.0.upgrade().map(WindowBufferModeDataRef)
    }
}

#[derive(Clone)]
pub struct WindowClientModeDataRef(Rc<UnsafeCell<window_client_modedata>>);

#[derive(Clone)]
pub struct WindowClientModeDataWeak(Weak<UnsafeCell<window_client_modedata>>);

impl WindowClientModeDataRef {
    pub(crate) fn new(value: window_client_modedata) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe {
            (*reference.as_ptr()).owner = Some(reference.downgrade());
        }
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut window_client_modedata {
        self.0.get()
    }

    pub(crate) fn downgrade(&self) -> WindowClientModeDataWeak {
        WindowClientModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowClientModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowClientModeDataRef> {
        self.0.upgrade().map(WindowClientModeDataRef)
    }
}

#[derive(Clone)]
pub struct WindowTreeModeDataRef(Rc<UnsafeCell<window_tree_modedata>>);

#[derive(Clone)]
pub struct WindowTreeModeDataWeak(Weak<UnsafeCell<window_tree_modedata>>);

impl WindowTreeModeDataRef {
    pub(crate) fn new(value: window_tree_modedata) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe {
            (*reference.as_ptr()).owner = Some(reference.downgrade());
        }
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut window_tree_modedata {
        self.0.get()
    }

    pub(crate) fn downgrade(&self) -> WindowTreeModeDataWeak {
        WindowTreeModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowTreeModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowTreeModeDataRef> {
        self.0.upgrade().map(WindowTreeModeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct WindowCustomizeModeDataRef(Rc<UnsafeCell<window_customize_modedata>>);

#[derive(Clone)]
pub struct WindowCustomizeModeDataWeak(Weak<UnsafeCell<window_customize_modedata>>);

impl WindowCustomizeModeDataRef {
    pub(crate) fn new(value: window_customize_modedata) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe {
            (*reference.as_ptr()).owner = Some(reference.downgrade());
        }
        reference
    }

    pub(crate) fn as_ptr(&self) -> *mut window_customize_modedata {
        self.0.get()
    }

    pub(crate) fn downgrade(&self) -> WindowCustomizeModeDataWeak {
        WindowCustomizeModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowCustomizeModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowCustomizeModeDataRef> {
        self.0.upgrade().map(WindowCustomizeModeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

pub type cmd_parse_status = ::core::ffi::c_uint;
#[repr(C)]
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub(crate) cmdlist: Option<CmdListRef>,
    pub error: Option<::std::ffi::CString>,
}
impl Default for cmd_parse_result {
    /// The `CMD_PARSE_SUCCESS` status with neither a command list nor an error.
    fn default() -> cmd_parse_result {
        cmd_parse_result {
            status: 0,
            cmdlist: None,
            error: None,
        }
    }
}
#[derive(Default)]
#[repr(C)]
pub enum CmdqCallbackData {
    #[default]
    None,
    NotifyEntry(Box<notify_entry>),
    WindowTreeModeData(WindowTreeModeDataWeak),
    String(::std::ffi::CString),
    KeyEvent(Box<key_event>),
}

pub type CmdqCallbackFn = unsafe fn(*mut cmdq_item, CmdqCallbackData) -> cmd_retval;

pub type cmdq_cb = Option<CmdqCallbackFn>;
pub use crate::style::colour_palette;
pub type control_sub_type = ::core::ffi::c_uint;
pub type size_t = usize;
pub type format_entry_cb = Option<unsafe fn(&format_tree) -> Option<::std::ffi::CString>>;
pub type gid_t = __gid_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct stat_t {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: ::core::ffi::c_int,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct glob_t {
    pub gl_pathc: __size_t,
    pub gl_pathv: *mut *mut ::core::ffi::c_char,
    pub gl_offs: __size_t,
    pub gl_flags: ::core::ffi::c_int,
    pub gl_closedir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub gl_readdir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut dirent>,
    pub gl_opendir:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> *mut ::core::ffi::c_void>,
    pub gl_lstat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat_t) -> ::core::ffi::c_int>,
    pub gl_stat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat_t) -> ::core::ffi::c_int>,
}
pub type u_char = __u_char;
pub type u_short = __u_short;
pub use crate::text::utf8_data;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct grid_cell {
    pub data: utf8_data,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
pub use crate::text::utf8_char;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_extd_entry {
    pub data: utf8_char,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
pub use crate::text::hanguljamo_state;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct imsg_hdr {
    pub type_0: uint32_t,
    pub len: uint32_t,
    pub peerid: uint32_t,
    pub pid: uint32_t,
}
pub type pid_t = __pid_t;
#[repr(C)]
pub struct imsgbuf {
    pub w: Option<Box<msgbuf>>,
    pub pid: pid_t,
    pub maxsize: uint32_t,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
pub type in_addr_t = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type uint16_t = __uint16_t;
pub type in_port_t = uint16_t;
/// What a client read back off the terminal, for the parser that asked.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum InputRequestData {
    #[default]
    None,
    Palette {
        idx: ::core::ffi::c_int,
        c: ::core::ffi::c_int,
    },
    Clipboard {
        clip: ::core::ffi::c_char,
        data: Vec<u8>,
    },
}
pub type input_request_type = ::core::ffi::c_uint;
/// The requests one parser is waiting on a reply to, oldest first. A request
/// belongs to the parser that made it until `input_free_request` takes it off.
pub type input_request_list = ::std::vec::Vec<::std::boxed::Box<input_request>>;
/// The requests one client is answering, oldest first, as the borrowed view
/// the replies walk. The parser that made each one owns it.
pub type input_requests = ::std::vec::Vec<*mut input_request>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct iovec {
    pub iov_base: *mut ::core::ffi::c_void,
    pub iov_len: size_t,
}
pub type job_complete_cb = Option<unsafe fn(*mut job) -> ()>;
pub type job_free_cb = Option<unsafe fn(JobData) -> ()>;
pub type job_update_cb = Option<unsafe fn(*mut job) -> ()>;
pub use crate::text::{key_code, key_code_type};
pub type tty_code_type = ::core::ffi::c_uint;
/// A mouse event. The default is an invalid one, which is what a client
/// starts out holding.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct mouse_event {
    pub valid: ::core::ffi::c_int,
    pub ignore: ::core::ffi::c_int,
    pub key: key_code,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub x: u_int,
    pub y: u_int,
    pub b: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
    pub ox: u_int,
    pub oy: u_int,
    pub s: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
    pub wp: ::core::ffi::c_int,
    pub sgr_type: u_int,
    pub sgr_b: u_int,
}
#[derive(Clone)]
#[repr(C)]
#[derive(Default)]
pub struct key_event {
    pub key: key_code,
    pub m: mouse_event,
    pub buf: Vec<u8>,
}
pub type layout_type = ::core::ffi::c_uint;
/// One item of a menu template. The default is the separator item: no name,
/// no key and no command.
/// One item a caller hands to `menu_add_item`. Its strings are borrowed:
/// from the `static` template a mode keeps, or from whatever the command
/// that is building the menu owns.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct menu_item<'a> {
    /// What the item reads as, which is expanded as a format before it is
    /// drawn, or nothing for the separator line.
    pub name: Option<&'a ::core::ffi::CStr>,
    pub key: key_code,
    /// The command the item runs, or nothing for one whose caller runs its
    /// own.
    pub command: Option<&'a ::core::ffi::CStr>,
}

/// One line of a menu as the menu itself holds it, which is not the same
/// thing as the [`menu_item`] a caller hands to `menu_add_item`: a template's
/// strings are borrowed — often from a `static` array — while a stored
/// entry's name is the expansion the menu made and owns.
#[repr(C)]
pub struct menu_entry {
    pub name: Option<::std::ffi::CString>,
    pub key: key_code,
    pub command: Option<::std::ffi::CString>,
}
impl Default for menu_entry {
    /// The separator line: no name, no key and no command.
    fn default() -> menu_entry {
        menu_entry {
            name: None,
            key: 0,
            command: None,
        }
    }
}
#[repr(C)]
pub struct menu {
    pub title: Option<::std::ffi::CString>,
    pub items: Vec<menu_entry>,
    pub width: u_int,
}
#[derive(Default)]
#[repr(C)]
pub enum MenuCallbackData {
    #[default]
    None,
    ModeTreeMenu(Box<mode_tree_menu>),
    StatusPromptMenu(Box<status_prompt_menu>),
    Popup(PopupDataWeak),
}
pub type menu_choice_cb = Option<unsafe fn(*mut menu, u_int, key_code, MenuCallbackData) -> ()>;
pub type sort_order = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sort_criteria_t {
    pub order: sort_order,
    pub reversed: ::core::ffi::c_int,
    pub order_seq: Option<&'static [sort_order]>,
}
impl Default for sort_criteria_t {
    /// Sorting by activity, forwards, with no explicit order sequence.
    fn default() -> sort_criteria_t {
        sort_criteria_t {
            order: crate::sort::SORT_ACTIVITY,
            reversed: 0,
            order_seq: None,
        }
    }
}
pub type uint64_t = __uint64_t;
pub type mode_tree_build_cb = Option<
    unsafe fn(WindowModeData, &sort_criteria_t, &mut uint64_t, *const ::core::ffi::c_char) -> (),
>;
pub type mode_tree_height_cb = Option<fn(&mut mode_tree_data, u_int) -> u_int>;
/// The help a mode shows: its own lines, the least width they need, and the
/// word `%1` stands for in every line.
pub type mode_tree_help_cb = Option<
    fn() -> (
        &'static [&'static ::core::ffi::CStr],
        u_int,
        &'static ::core::ffi::CStr,
    ),
>;
pub type mode_tree_key_cb = Option<unsafe fn(WindowModeData, ModeTreeItemData, u_int) -> key_code>;
pub type mode_tree_search_cb = Option<
    unsafe fn(
        WindowModeData,
        ModeTreeItemData,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub type mode_tree_sort_cb = Option<fn(&mut sort_criteria_t) -> ()>;
pub type mode_tree_swap_cb =
    Option<unsafe fn(ModeTreeItemData, ModeTreeItemData, &sort_criteria_t) -> ::core::ffi::c_int>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_command {
    pub argc: ::core::ffi::c_int,
}
pub type msgtype = ::core::ffi::c_uint;
pub type nl_item = ::core::ffi::c_int;
/// The values of an array option, by index, so that the indexes need not run
/// without gaps.
pub type options_array = ::std::collections::BTreeMap<u_int, options_array_item_t>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_name_map {
    pub from: &'static ::core::ffi::CStr,
    pub to: &'static ::core::ffi::CStr,
}
pub type options_table_type = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_table_entry_t {
    pub name: &'static ::core::ffi::CStr,
    pub alternative_name: Option<&'static ::core::ffi::CStr>,
    pub type_0: options_table_type,
    pub scope: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: Option<&'static [&'static ::core::ffi::CStr]>,
    pub default_str: Option<&'static ::core::ffi::CStr>,
    pub default_num: ::core::ffi::c_longlong,
    pub default_arr: Option<&'static [&'static ::core::ffi::CStr]>,
    pub separator: Option<&'static ::core::ffi::CStr>,
    pub pattern: Option<&'static ::core::ffi::CStr>,
    pub text: Option<&'static ::core::ffi::CStr>,
    pub unit: Option<&'static ::core::ffi::CStr>,
}
pub type style_align = ::core::ffi::c_uint;
pub type style_default_type = ::core::ffi::c_uint;
pub type style_list = ::core::ffi::c_uint;
pub type style_range_type = ::core::ffi::c_uint;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct style {
    pub gc: grid_cell,
    pub ignore: ::core::ffi::c_int,
    pub fill: ::core::ffi::c_int,
    pub align: style_align,
    pub list: style_list,
    pub range_type: style_range_type,
    pub range_argument: u_int,
    pub range_string: [::core::ffi::c_char; 16],
    pub width: ::core::ffi::c_int,
    pub width_percentage: ::core::ffi::c_int,
    pub pad: ::core::ffi::c_int,
    pub default_type: style_default_type,
}
/// What one value of an option holds. The kind the option's table entry
/// names is what it is set to; nothing else is ever read out of it.
#[derive(Default)]
#[repr(C)]
pub enum options_value {
    /// A value the option has not been given yet.
    #[default]
    None,
    Number(::core::ffi::c_longlong),
    String(::std::ffi::CString),
    Commands(CmdListRef),
}

impl options_value {
    /// The number the value holds, or zero when it holds something else.
    pub fn number(&self) -> ::core::ffi::c_longlong {
        match self {
            options_value::Number(number) => *number,
            _ => 0,
        }
    }

    /// The string the value holds, which every caller of this has already
    /// established it is.
    pub fn string(&self) -> &::core::ffi::CStr {
        match self {
            options_value::String(string) => string,
            _ => panic!("not a string option"),
        }
    }

    /// The command list the value holds, or null when it holds something
    /// else.
    pub fn cmdlist(&self) -> *mut cmd_list {
        match self {
            options_value::Commands(cmdlist) => cmdlist.as_ptr(),
            _ => ::core::ptr::null_mut(),
        }
    }

    /// The command list the value holds as a handle, so a caller can keep it.
    pub(crate) fn commands(&self) -> Option<CmdListRef> {
        match self {
            options_value::Commands(cmdlist) => Some(cmdlist.clone()),
            _ => None,
        }
    }
}
pub type pane_lines = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct passwd {
    pub pw_name: *mut ::core::ffi::c_char,
    pub pw_passwd: *mut ::core::ffi::c_char,
    pub pw_uid: __uid_t,
    pub pw_gid: __gid_t,
    pub pw_gecos: *mut ::core::ffi::c_char,
    pub pw_dir: *mut ::core::ffi::c_char,
    pub pw_shell: *mut ::core::ffi::c_char,
}
pub type popup_finish_edit_cb =
    Option<unsafe fn(::std::vec::Vec<u8>, ::std::boxed::Box<window_buffer_editdata>) -> ()>;
pub type progress_bar_state = ::core::ffi::c_uint;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct progress_bar {
    pub state: progress_bar_state,
    pub progress: ::core::ffi::c_int,
}
pub type prompt_type = ::core::ffi::c_uint;
pub type reg_syntax_t = ::core::ffi::c_ulong;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct re_pattern_buffer {
    pub buffer: *mut re_dfa_t,
    pub allocated: __re_long_size_t,
    pub used: __re_long_size_t,
    pub syntax: reg_syntax_t,
    pub fastmap: *mut ::core::ffi::c_char,
    pub translate: *mut ::core::ffi::c_uchar,
    pub re_nsub: size_t,
    #[bitfield(name = "can_be_null", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "regs_allocated", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "fastmap_accurate", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "no_sub", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "not_bol", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "not_eol", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "newline_anchor", ty = "::core::ffi::c_uint", bits = "7..=7")]
    pub can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
/// The pattern buffer `regcomp` expects to be handed, which is the all-zero
/// one: the compile fills in every field it uses. `Default` cannot be derived
/// because `buffer` points at an extern type.
impl Default for re_pattern_buffer {
    fn default() -> Self {
        Self {
            buffer: ::core::ptr::null_mut(),
            allocated: 0,
            used: 0,
            syntax: 0,
            fastmap: ::core::ptr::null_mut(),
            translate: ::core::ptr::null_mut(),
            re_nsub: 0,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0;
                1],
            c2rust_padding: [0; 7],
        }
    }
}
pub type regex_t = re_pattern_buffer;
pub type regoff_t = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}
pub type sa_family_t = ::core::ffi::c_ushort;
pub type screen_cursor_style = ::core::ffi::c_uint;
pub type sigset_t = __sigset_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [::core::ffi::c_uchar; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [::core::ffi::c_char; 108],
}
impl Default for sockaddr_un {
    /// An address in no family, with an empty path.
    fn default() -> sockaddr_un {
        sockaddr_un {
            sun_family: 0,
            sun_path: [0; 108],
        }
    }
}
pub type socklen_t = __socklen_t;
pub type speed_t = ::core::ffi::c_uint;
pub type ssize_t = isize;
pub type tcflag_t = ::core::ffi::c_uint;
pub type time_t = __time_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
impl timeval {
    /// A span of whole seconds.
    pub const fn from_secs(tv_sec: __time_t) -> timeval {
        timeval { tv_sec, tv_usec: 0 }
    }
    /// A span of microseconds, which the callers here keep under a second.
    pub const fn from_usecs(tv_usec: __suseconds_t) -> timeval {
        timeval { tv_sec: 0, tv_usec }
    }
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct tm {
    pub tm_sec: ::core::ffi::c_int,
    pub tm_min: ::core::ffi::c_int,
    pub tm_hour: ::core::ffi::c_int,
    pub tm_mday: ::core::ffi::c_int,
    pub tm_mon: ::core::ffi::c_int,
    pub tm_year: ::core::ffi::c_int,
    pub tm_wday: ::core::ffi::c_int,
    pub tm_yday: ::core::ffi::c_int,
    pub tm_isdst: ::core::ffi::c_int,
    pub tm_gmtoff: ::core::ffi::c_long,
    pub tm_zone: *const ::core::ffi::c_char,
}
pub type tty_code_code = ::core::ffi::c_uint;
pub type uid_t = __uid_t;
pub type uint8_t = __uint8_t;
pub use crate::text::utf8_state;
pub type va_list = __builtin_va_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visible_range {
    pub px: u_int,
    pub nx: u_int,
}
#[derive(Default)]
#[repr(C)]
pub struct visible_ranges {
    pub ranges: Vec<visible_range>,
    pub used: u_int,
}
pub type wchar_t = ::libc::wchar_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct window_pane_offset {
    pub used: size_t,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_read_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_cancel {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_write_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_write_ready {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_close {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_code {
    pub code: tty_code_code,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_raw {
    /// The bytes the terminal sends for the key.
    pub string: &'static ::core::ffi::CStr,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_xterm {
    /// The bytes the terminal sends for the key, with `_` standing where the
    /// modifier digit goes.
    pub template: &'static ::core::ffi::CStr,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    /// The terminfo name of the capability.
    pub name: &'static ::core::ffi::CStr,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_cell {
    pub cell: grid_cell,
    pub set: ::core::ffi::c_int,
    pub g0set: ::core::ffi::c_int,
    pub g1set: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_table_entry {
    pub ch: ::core::ffi::c_int,
    /// The intermediate bytes the sequence carries before its final byte.
    pub interm: &'static ::core::ffi::CStr,
    pub type_0: ::core::ffi::c_int,
}

#[derive(Clone)]
#[repr(C)]
pub struct cmd_command_prompt_prompt {
    pub input: Option<::std::ffi::CString>,
    pub prompt: Option<::std::ffi::CString>,
}
#[derive(Clone, Default)]
#[repr(C)]
pub struct window_buffer_itemdata {
    pub name: Option<::std::ffi::CString>,
    pub order: u_int,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union __SOCKADDR_ARG {
    pub __sockaddr__: *mut sockaddr,
    pub __sockaddr_at__: *mut sockaddr_at,
    pub __sockaddr_ax25__: *mut sockaddr_ax25,
    pub __sockaddr_dl__: *mut sockaddr_dl,
    pub __sockaddr_eon__: *mut sockaddr_eon,
    pub __sockaddr_in__: *mut sockaddr_in,
    pub __sockaddr_in6__: *mut sockaddr_in6,
    pub __sockaddr_inarp__: *mut sockaddr_inarp,
    pub __sockaddr_ipx__: *mut sockaddr_ipx,
    pub __sockaddr_iso__: *mut sockaddr_iso,
    pub __sockaddr_ns__: *mut sockaddr_ns,
    pub __sockaddr_un__: *mut sockaddr_un,
    pub __sockaddr_x25__: *mut sockaddr_x25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union __CONST_SOCKADDR_ARG {
    pub __sockaddr__: *const sockaddr,
    pub __sockaddr_at__: *const sockaddr_at,
    pub __sockaddr_ax25__: *const sockaddr_ax25,
    pub __sockaddr_dl__: *const sockaddr_dl,
    pub __sockaddr_eon__: *const sockaddr_eon,
    pub __sockaddr_in__: *const sockaddr_in,
    pub __sockaddr_in6__: *const sockaddr_in6,
    pub __sockaddr_inarp__: *const sockaddr_inarp,
    pub __sockaddr_ipx__: *const sockaddr_ipx,
    pub __sockaddr_iso__: *const sockaddr_iso,
    pub __sockaddr_ns__: *const sockaddr_ns,
    pub __sockaddr_un__: *const sockaddr_un,
    pub __sockaddr_x25__: *const sockaddr_x25,
}
#[repr(C)]
pub struct format_modifier {
    pub modifier: [::core::ffi::c_char; 3],
    pub size: u_int,
    pub argv: Vec<::std::ffi::CString>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_cell_entry_data {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union grid_cell_entry_union {
    pub offset: u_int,
    pub data: grid_cell_entry_data,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_cell_entry {
    pub c2rust_unnamed: grid_cell_entry_union,
    pub flags: u_char,
}
const _: () = assert!(::core::mem::size_of::<grid_cell_entry>() == 5);
#[repr(C)]
pub struct grid {
    pub flags: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub hscrolled: u_int,
    pub hsize: u_int,
    pub hlimit: u_int,
    pub linedata: Vec<grid_line>,
}
pub use crate::grid::grid_line;
/// A cursor walking a grid. It borrows the grid, so a walk cannot outlive
/// the lines it is reading.
pub struct grid_reader<'a> {
    pub gd: &'a grid,
    pub cx: u_int,
    pub cy: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_range {
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [::core::ffi::c_char; 16],
    pub start: u_int,
    pub end: u_int,
}
pub type style_ranges = ::std::vec::Vec<style_range>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resize_t {
    pub sx: u_int,
    pub sy: u_int,
    pub osx: u_int,
    pub osy: u_int,
}
pub type window_pane_resizes = ::std::collections::VecDeque<window_pane_resize_t>;
#[derive(Default)]
pub struct style_line_entry {
    pub expanded: Option<::std::ffi::CString>,
    pub ranges: style_ranges,
}
#[repr(C)]
pub struct screen {
    pub title: Option<::std::ffi::CString>,
    pub path: Option<::std::ffi::CString>,
    pub titles: Option<Box<screen_titles>>,
    pub ntitles: u_int,
    pub grid: Option<Box<grid>>,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub default_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub default_ccolour: ::core::ffi::c_int,
    pub rupper: u_int,
    pub rlower: u_int,
    pub mode: ::core::ffi::c_int,
    pub default_mode: ::core::ffi::c_int,
    pub saved_cx: u_int,
    pub saved_cy: u_int,
    pub saved_grid: Option<Box<grid>>,
    pub saved_cell: grid_cell,
    pub saved_flags: ::core::ffi::c_int,
    pub tabs: Vec<u8>,
    pub sel: Option<Box<screen_sel>>,
    pub write_list: Vec<screen_write_cline>,
    pub(crate) hyperlinks: Option<HyperlinksRef>,
    pub progress_bar: progress_bar,
}
pub type client_exit_type = ::core::ffi::c_uint;
pub type client_prompt_mode = ::core::ffi::c_uint;
#[repr(C)]
pub struct winlink {
    pub idx: ::core::ffi::c_int,
    /// The session that holds the link, observed rather than held: the
    /// session owns the link, so holding it back would be a cycle.
    pub(crate) session_ref: Option<SessionWeak>,
    pub(crate) window_ref: Option<WindowRef>,
    pub flags: ::core::ffi::c_int,
}

impl winlink {
    /// The session that holds this link, or null once it has gone.
    pub(crate) fn session(&self) -> *mut session {
        self.session_ref
            .as_ref()
            .and_then(SessionWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr())
    }

    /// Records `s` as the session that holds this link.
    pub(crate) fn set_session(&mut self, s: *mut session) {
        self.session_ref = crate::session::session_ref_from_ptr(s).map(|s| s.downgrade());
    }

    /// The window this link points at, or null while the link holds none.
    pub(crate) fn window(&self) -> *mut window {
        self.window_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), WindowRef::as_ptr)
    }

    /// The handle on the window this link points at, borrowed for as long as
    /// the link is, or nothing while the link holds none.
    pub(crate) fn window_handle(&self) -> Option<&WindowRef> {
        self.window_ref.as_ref()
    }
}
#[derive(Default)]
#[repr(C)]
pub struct window {
    pub id: u_int,
    pub(crate) latest: Option<ClientWeak>,
    pub name: Option<::std::ffi::CString>,
    pub name_event: TimerHandle,
    pub name_time: timeval,
    pub alerts_timer: TimerHandle,
    pub offset_timer: TimerHandle,
    pub activity_time: timeval,
    pub creation_time: timeval,
    /// The id of the pane the window is showing as its active one, or
    /// nothing while it has none. A pane is named by its id and nothing
    /// else, so the window never holds one that has been destroyed.
    pub active_id: Option<u_int>,
    pub last_panes: window_pane_stack_t,
    pub z_index: window_pane_stack_t,
    pub panes: window_panes_t,
    pub lastlayout: ::core::ffi::c_int,
    pub layout_root: Option<::std::boxed::Box<layout_cell>>,
    pub saved_layout_root: Option<::std::boxed::Box<layout_cell>>,
    pub old_layout: Option<::std::ffi::CString>,
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
    pub last_new_pane_x: u_int,
    pub last_new_pane_y: u_int,
    pub sb: ::core::ffi::c_int,
    pub sb_pos: ::core::ffi::c_int,
    pub fill_character: Option<Box<utf8_data>>,
    pub flags: ::core::ffi::c_int,
    pub alerts_queued: ::core::ffi::c_int,
    pub options: Option<Box<options>>,
    /// The sessions' links to this window, in the order they were made. A
    /// link belongs to its session, not to the window.
    pub(crate) winlinks: window_winlinks,
}
impl window {
    /// The window's own option set, or null for a window that carries none.
    pub(crate) fn options_ptr(&self) -> *mut options {
        crate::options::options_ptr(&self.options)
    }

    /// The root cell of the window's layout, or null for a window that has
    /// none.
    pub(crate) fn layout_root_ptr(&self) -> *mut layout_cell {
        crate::layout::layout_root_ptr(&self.layout_root)
    }

    /// The root cell of the layout the window had before a pane in it was
    /// zoomed, or null for a window that is not zoomed.
    pub(crate) fn saved_layout_root_ptr(&self) -> *mut layout_cell {
        crate::layout::layout_root_ptr(&self.saved_layout_root)
    }
}
#[derive(Default)]
#[repr(C)]
pub struct window_pane {
    pub id: u_int,
    pub active_point: u_int,
    /// The window that owns this pane. A borrow, not an owning edge: the
    /// window holds its panes by value, so a pane never outlives it.
    pub window: *mut window,
    pub options: Option<Box<options>>,
    pub layout_cell: *mut layout_cell,
    pub saved_layout_cell: *mut layout_cell,
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub sb_slider_y: u_int,
    pub sb_slider_h: u_int,
    pub argv: Vec<::std::ffi::CString>,
    pub shell: Option<::std::ffi::CString>,
    pub cwd: Option<::std::ffi::CString>,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub dead_time: timeval,
    pub fd: ::core::ffi::c_int,
    pub event: Stream,
    pub offset: window_pane_offset,
    pub base_offset: size_t,
    pub resize_queue: window_pane_resizes,
    pub resize_timer: TimerHandle,
    pub sync_timer: TimerHandle,
    pub ictx: Option<InputCtxRef>,
    pub cached_gc: grid_cell,
    pub cached_active_gc: grid_cell,
    pub palette: colour_palette,
    pub last_theme: client_theme,
    pub border_status_line: style_line_entry,
    pub pipe_fd: ::core::ffi::c_int,
    pub pipe_pid: pid_t,
    pub pipe_event: Stream,
    pub pipe_offset: window_pane_offset,
    /// Which screen the pane is showing: its own, or the one the mode at the
    /// front of its mode list draws on.
    pub(crate) shown: PaneScreen,
    pub base: screen,
    pub status_screen: screen,
    pub status_size: size_t,
    pub modes: window_modes,
    pub searchstr: Option<::std::ffi::CString>,
    pub searchregex: ::core::ffi::c_int,
    pub border_gc_set: ::core::ffi::c_int,
    pub border_gc: grid_cell,
    pub active_border_gc_set: ::core::ffi::c_int,
    pub active_border_gc: grid_cell,
    pub control_bg: ::core::ffi::c_int,
    pub control_fg: ::core::ffi::c_int,
    pub scrollbar_style: style,
    pub r: visible_ranges,
}

/// Which screen a pane is showing.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum PaneScreen {
    /// The pane's own screen, which it holds itself.
    #[default]
    Base,
    /// The screen the mode at the front of the pane's mode list draws on.
    Mode,
}

impl window_pane {
    /// The pane's own option set, or null for a pane that carries none.
    pub(crate) fn options_ptr(&self) -> *mut options {
        crate::options::options_ptr(&self.options)
    }

    /// The screen the pane is showing. A pane showing a mode falls back to
    /// its own screen when the mode list has run out.
    pub fn screen(&self) -> *mut screen {
        let base = &raw const self.base as *mut screen;
        match self.shown {
            PaneScreen::Base => base,
            PaneScreen::Mode => match self.modes.first() {
                Some(wme) => wme.screen,
                None => base,
            },
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode {
    /// The mode's name, as `#{pane_mode}` and `choose-tree` report it.
    pub name: &'static ::core::ffi::CStr,
    /// The format the mode draws its lines with, or nothing for a mode that
    /// has none of its own.
    pub default_format: Option<&'static ::core::ffi::CStr>,
    pub init:
        Option<unsafe fn(&mut window_mode_entry, *mut cmd_find_state, *mut args) -> *mut screen>,
    pub free: Option<unsafe fn(&mut window_mode_entry) -> ()>,
    pub resize: Option<unsafe fn(&mut window_mode_entry, u_int, u_int) -> ()>,
    pub update: Option<unsafe fn(&mut window_mode_entry) -> ()>,
    pub style_changed: Option<unsafe fn(&mut window_mode_entry) -> ()>,
    pub key: Option<
        unsafe fn(
            &mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            key_code,
            *mut mouse_event,
        ) -> (),
    >,
    pub key_table: Option<unsafe fn(&mut window_mode_entry) -> &'static ::core::ffi::CStr>,
    pub command: Option<
        unsafe fn(
            &mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            *mut args,
            *mut mouse_event,
        ) -> (),
    >,
    pub formats: Option<unsafe fn(&mut window_mode_entry, &mut format_tree) -> ()>,
    pub get_screen: Option<unsafe fn(&mut window_mode_entry) -> *mut screen>,
}
/// Which logical mode a pane is in, and the selector for the [`window_mode`]
/// table that mode dispatches through.
///
/// The tables live in `impl WindowMode`, in the `window_mode` module; every
/// one of them is reachable from [`WindowMode::table`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum WindowMode {
    Clock,
    Copy,
    View,
    Buffer,
    Client,
    Tree,
    Customize,
}
/// A borrowed raw-pointer view of the private state a mode's
/// [`WindowMode::init`] built.
///
/// The mode a tree belongs to, observed rather than held: the mode owns the
/// tree, so the tree names its mode this way and finds nothing once the mode
/// has gone.
#[derive(Clone, Default)]
#[repr(C)]
pub enum WindowModeData {
    #[default]
    None,
    Buffer(WindowBufferModeDataWeak),
    Client(WindowClientModeDataWeak),
    Tree(WindowTreeModeDataWeak),
    Customize(WindowCustomizeModeDataWeak),
}
/// The complete state of a window-mode entry.
///
/// The variant is the logical mode and its payload owns the mode's private
/// state. The raw-pointer [`WindowModeData`] form is derived from this value
/// when code using the translated callback interfaces needs it.
#[derive(Default)]
pub(crate) enum WindowModeState {
    #[default]
    None,
    Clock(Box<window_clock_mode_data>),
    Copy(Box<window_copy_mode_data>),
    View(Box<window_copy_mode_data>),
    Buffer(WindowBufferModeDataRef),
    Client(WindowClientModeDataRef),
    Tree(WindowTreeModeDataRef),
    Customize(WindowCustomizeModeDataRef),
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub enum ModeTreeItemData {
    #[default]
    None,
    Buffer(*mut window_buffer_itemdata),
    Client(*mut window_client_itemdata),
    Tree(*mut window_tree_itemdata),
    Customize(*mut window_customize_itemdata),
}
impl ModeTreeItemData {
    pub fn buffer(self) -> *mut window_buffer_itemdata {
        match self {
            ModeTreeItemData::Buffer(data) => data,
            ModeTreeItemData::None => ::core::ptr::null_mut(),
            _ => panic!("not buffer-mode item data"),
        }
    }

    pub fn client(self) -> *mut window_client_itemdata {
        match self {
            ModeTreeItemData::Client(data) => data,
            ModeTreeItemData::None => ::core::ptr::null_mut(),
            _ => panic!("not client-mode item data"),
        }
    }

    pub fn tree(self) -> *mut window_tree_itemdata {
        match self {
            ModeTreeItemData::Tree(data) => data,
            ModeTreeItemData::None => ::core::ptr::null_mut(),
            _ => panic!("not tree-mode item data"),
        }
    }

    pub fn customize(self) -> *mut window_customize_itemdata {
        match self {
            ModeTreeItemData::Customize(data) => data,
            ModeTreeItemData::None => ::core::ptr::null_mut(),
            _ => panic!("not customize-mode item data"),
        }
    }
}
#[repr(C)]
pub struct window_mode_entry {
    pub wp: *mut window_pane,
    pub swp: *mut window_pane,
    pub(crate) state: WindowModeState,
    pub screen: *mut screen,
    pub prefix: u_int,
    pub(crate) mode_tree_ref: Option<ModeTreeDataRef>,
}
#[derive(Default)]
#[repr(C)]
pub struct layout_cell {
    pub type_0: layout_type,
    pub flags: ::core::ffi::c_int,
    pub parent: *mut layout_cell,
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    /// The id of the pane the cell holds, or nothing for a cell that holds
    /// other cells instead. A pane is named by its id and nothing else, so a
    /// cell never holds one that has been destroyed.
    pub wp_id: Option<u_int>,
    pub cells: layout_cells,
}
#[derive(Default)]
#[repr(C)]
pub struct client {
    pub name: Option<::std::ffi::CString>,
    pub peer: Option<Box<tmuxpeer>>,
    pub user: Option<::std::ffi::CString>,
    pub queue: Option<Box<cmdq_list>>,
    pub windows: client_windows,
    pub control_state: Option<Box<control_state>>,
    pub pause_age: u_int,
    pub pid: pid_t,
    pub fd: ::core::ffi::c_int,
    pub out_fd: ::core::ffi::c_int,
    pub event: IoHandle,
    pub retval: ::core::ffi::c_int,
    pub creation_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub environ: Option<Box<environ_t>>,
    pub jobs: Option<Box<format_job_tree>>,
    pub title: Option<::std::ffi::CString>,
    pub path: Option<::std::ffi::CString>,
    pub cwd: Option<::std::ffi::CString>,
    pub progress_bar: progress_bar,
    pub term_name: Option<::std::ffi::CString>,
    pub term_features: ::core::ffi::c_int,
    pub term_type: Option<::std::ffi::CString>,
    pub term_caps: Vec<::std::ffi::CString>,
    pub ttyname: Option<::std::ffi::CString>,
    pub tty: tty,
    pub written: size_t,
    pub discarded: size_t,
    pub redraw: size_t,
    pub repeat_timer: TimerHandle,
    pub click_timer: TimerHandle,
    pub click_loc: ::core::ffi::c_int,
    pub click_wp: ::core::ffi::c_int,
    pub click_button: u_int,
    pub click_event: mouse_event,
    pub status: status_line,
    pub theme: client_theme,
    pub input_requests: input_requests,
    pub flags: uint64_t,
    pub exit_type: client_exit_type,
    pub exit_msgtype: msgtype,
    pub exit_session: Option<::std::ffi::CString>,
    pub exit_message: Option<::std::ffi::CString>,
    pub(crate) keytable_ref: Option<KeyTableRef>,
    pub last_key: key_code,
    pub paste_time: time_t,
    pub redraw_panes: uint64_t,
    pub redraw_scrollbars: uint64_t,
    pub message_ignore_keys: ::core::ffi::c_int,
    pub message_ignore_styles: ::core::ffi::c_int,
    pub message_string: Option<::std::ffi::CString>,
    pub(crate) message_overlay: Option<StatusScreenRef>,
    pub message_timer: TimerHandle,
    pub prompt_string: Option<::std::ffi::CString>,
    pub(crate) prompt_overlay: Option<StatusScreenRef>,
    pub prompt_buffer: Vec<utf8_data>,
    pub prompt_state: cmd_find_state,
    pub prompt_last: Option<::std::ffi::CString>,
    pub prompt_index: size_t,
    pub prompt: Prompt,
    pub prompt_data: PromptData,
    pub prompt_hindex: [u_int; 4],
    pub prompt_mode: client_prompt_mode,
    pub prompt_saved: Option<Vec<utf8_data>>,
    pub prompt_flags: ::core::ffi::c_int,
    pub prompt_type: prompt_type,
    pub prompt_cursor: ::core::ffi::c_int,
    pub session: *mut session,
    pub(crate) last_session: Option<SessionWeak>,
    pub(crate) pan_window: Option<WindowWeak>,
    pub pan_ox: u_int,
    pub pan_oy: u_int,
    /// The overlay the client is showing, what it is showing it with, and
    /// which of the two answers for the region the current drawing may
    /// cover. All four are reached from outside this module through the
    /// `overlay*` methods below: `server_client_set_overlay` and
    /// `server_client_clear_overlay` put an overlay up and take it down, and
    /// a popup retargets the region with `set_overlay_view` while it draws.
    overlay_check: OverlayCheck,
    overlay: Overlay,
    overlay_data: OverlayState,
    overlay_data_view: Option<OverlayData>,
    pub overlay_timer: TimerHandle,
    pub(crate) files: client_files_t,
    pub source_file_depth: u_int,
    pub clipboard_npanes: u_int,
}
#[repr(C)]
pub struct client_file {
    pub(crate) client_ref: Option<ClientRef>,
    pub peer: *mut tmuxpeer,
    /// Which set of files this one belongs to.
    pub(crate) tree: FileOwner,
    pub stream: ::core::ffi::c_int,
    pub path: Option<::std::ffi::CString>,
    pub buffer: Box<Buf>,
    pub event: Stream,
    pub fd: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
    pub closed: ::core::ffi::c_int,
    pub done: ::core::ffi::c_int,
    pub cb: client_file_cb,
    pub data: ClientFileData,
}

impl client_file {
    /// The client the file belongs to, or null for one opened against a peer
    /// alone.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }
}
#[derive(Default)]
#[repr(C)]
pub enum ClientFileData {
    #[default]
    None,
    LoadBuffer(Box<cmd_load_buffer_data>),
    LoadBufferView(*mut cmd_load_buffer_data),
    SaveBuffer(*mut cmdq_item),
    SourceFile(SourceFileRef),
    PaneInput(PaneInputRef),
}

impl ClientFileData {
    pub(crate) fn view(&self) -> ClientFileData {
        match self {
            ClientFileData::None => ClientFileData::None,
            ClientFileData::LoadBuffer(data) => ClientFileData::LoadBufferView(data.as_ref()
                as *const cmd_load_buffer_data
                as *mut cmd_load_buffer_data),
            ClientFileData::LoadBufferView(data) => ClientFileData::LoadBufferView(*data),
            ClientFileData::SaveBuffer(data) => ClientFileData::SaveBuffer(*data),
            ClientFileData::SourceFile(data) => ClientFileData::SourceFile(data.clone()),
            ClientFileData::PaneInput(data) => ClientFileData::PaneInput(data.clone()),
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_window {
    pub window: u_int,
    /// The id of the pane the client has made active in this window, or
    /// nothing while it has made none. A pane is named by its id and nothing
    /// else, so the entry never names one that has been destroyed.
    pub pane_id: Option<u_int>,
    pub sx: u_int,
    pub sy: u_int,
}
/// A resolved command target. The default is an unresolved one: no session,
/// window or pane, and no flags.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_find_state {
    pub flags: ::core::ffi::c_int,
    /// The session the state found, observed rather than held, so that a
    /// state kept across a queue turn finds nothing rather than freed memory.
    pub(crate) s_ref: Option<SessionWeak>,
    /// The index of the link the state found, resolved against the session
    /// above, or nothing when it found none.
    pub wl_idx: Option<::core::ffi::c_int>,
    /// The window the state found, observed the same way.
    pub(crate) w_ref: Option<WindowWeak>,
    /// The id of the pane the state found, or nothing when it found none. A
    /// pane is named by its id and nothing else, so a state that outlives the
    /// pane it found answers with nothing rather than with freed memory.
    pub wp_id: Option<u_int>,
    pub idx: ::core::ffi::c_int,
}

impl cmd_find_state {
    /// The session the state found, or null when it found none or the server
    /// has since given it up.
    pub fn session(&self) -> *mut session {
        self.s_ref
            .as_ref()
            .and_then(SessionWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr())
    }

    /// Records `s` as the session the state found.
    pub fn set_session(&mut self, s: *mut session) {
        self.s_ref = crate::session::session_ref_from_ptr(s).map(|s| s.downgrade());
    }

    /// The link the state found, or null when it found none or the session
    /// has since given it up.
    pub fn winlink(&self) -> *mut winlink {
        crate::session::winlink_of(self.session(), self.wl_idx)
    }

    /// Records `wl` as the link the state found.
    pub unsafe fn set_winlink(&mut self, wl: *mut winlink) {
        self.wl_idx = unsafe { wl.as_ref().map(|wl| wl.idx) };
    }

    /// The window the state found, or null the same way.
    pub fn window(&self) -> *mut window {
        self.w_ref
            .as_ref()
            .and_then(WindowWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |w| w.as_ptr())
    }

    /// Records `w` as the window the state found.
    pub fn set_window(&mut self, w: *mut window) {
        self.w_ref = crate::window::window_ref_from_ptr(w).map(|w| w.downgrade());
    }

    /// The pane the state found, or null when it found none or the server has
    /// since given that pane up.
    pub fn pane(&self) -> *mut window_pane {
        let Some(id) = self.wp_id else {
            return ::core::ptr::null_mut();
        };
        let w = self.window();
        match w.is_null() {
            true => crate::window::window_pane_find_by_id(id),
            false => crate::window::window_pane_of_id(w, id),
        }
    }

    /// Records `wp` as the pane the state found, or none when it is null.
    pub unsafe fn set_pane(&mut self, wp: *mut window_pane) {
        self.wp_id = unsafe { wp.as_ref().map(|wp| wp.id) };
    }
}
/// The state of one redraw. The default is a context for no client, with
/// every offset and size zero.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct screen_redraw_ctx {
    pub c: *mut client,
    pub statuslines: u_int,
    pub statustop: ::core::ffi::c_int,
    pub pane_status: ::core::ffi::c_int,
    pub pane_lines: pane_lines,
    pub pane_scrollbars: ::core::ffi::c_int,
    pub pane_scrollbars_pos: ::core::ffi::c_int,
    pub no_pane_gc: grid_cell,
    pub no_pane_gc_set: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ox: ::core::ffi::c_int,
    pub oy: ::core::ffi::c_int,
}
#[derive(Default)]
#[repr(C)]
pub struct status_line {
    pub timer: TimerHandle,
    pub screen: screen,
    /// Which screen the status line is drawn on: its own, or the overlay a
    /// message or prompt put in front of it.
    pub(crate) active: StatusActive,
    pub style: grid_cell,
    pub entries: [style_line_entry; 5],
}

/// Which screen a status line is drawing on.
#[derive(Default)]
pub(crate) enum StatusActive {
    /// The status line's own screen, which it holds itself.
    #[default]
    Own,
    /// The screen a message or prompt put in front of it, watched rather
    /// than held: the overlay slots are what keep it.
    Overlay(StatusScreenWeak),
}

impl status_line {
    /// The screen the status line is drawing on.
    pub(crate) fn active(&self) -> *mut screen {
        let own = &raw const self.screen as *mut screen;
        match &self.active {
            StatusActive::Own => own,
            StatusActive::Overlay(watched) => match watched.screen() {
                overlay if overlay.is_null() => own,
                overlay => overlay,
            },
        }
    }

    /// Whether the status line is drawing on its own screen.
    pub(crate) fn is_own(&self) -> bool {
        matches!(self.active, StatusActive::Own)
    }
}
#[repr(C)]
pub struct tty {
    /// The client whose terminal this is, observed rather than held: a tty
    /// is an inline field of its client, so holding it would be a cycle.
    pub(crate) owner: Option<ClientWeak>,
    pub start_timer: TimerHandle,
    pub clipboard_timer: TimerHandle,
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
    pub event_in: IoHandle,
    pub in_0: Option<Box<Buf>>,
    pub event_out: IoHandle,
    pub out: Option<Box<Buf>>,
    pub timer: TimerHandle,
    pub discarded: size_t,
    pub tio: termios,
    pub r: visible_ranges,
    pub cell: grid_cell,
    pub last_cell: grid_cell,
    pub flags: ::core::ffi::c_int,
    pub term: Option<Box<tty_term>>,
    pub mouse_last_x: u_int,
    pub mouse_last_y: u_int,
    pub mouse_last_b: u_int,
    pub mouse_drag_flag: ::core::ffi::c_int,
    pub mouse_scrolling_flag: ::core::ffi::c_int,
    pub mouse_slider_mpos: ::core::ffi::c_int,
    pub mouse_last_pane: ::core::ffi::c_int,
    pub mouse_drag_update: Option<unsafe fn(*mut client, *mut mouse_event) -> ()>,
    pub mouse_drag_release: Option<unsafe fn(*mut client, *mut mouse_event) -> ()>,
    pub key_timer: TimerHandle,
    pub key_tree: Option<Box<tty_key>>,
}
impl Default for tty {
    /// A terminal that has not been opened yet: no client, no events, no term.
    fn default() -> tty {
        tty {
            owner: None,
            start_timer: TimerHandle::default(),
            clipboard_timer: TimerHandle::default(),
            last_requests: 0,
            sx: 0,
            sy: 0,
            xpixel: 0,
            ypixel: 0,
            cx: 0,
            cy: 0,
            cstyle: 0,
            ccolour: 0,
            oflag: 0,
            oox: 0,
            ooy: 0,
            osx: 0,
            osy: 0,
            mode: 0,
            fg: 0,
            bg: 0,
            rlower: 0,
            rupper: 0,
            rleft: 0,
            rright: 0,
            event_in: IoHandle::default(),
            in_0: None,
            event_out: IoHandle::default(),
            out: None,
            timer: TimerHandle::default(),
            discarded: 0,
            tio: unsafe { ::core::mem::zeroed() },
            r: visible_ranges::default(),
            cell: grid_cell::default(),
            last_cell: grid_cell::default(),
            flags: 0,
            term: None,
            mouse_last_x: 0,
            mouse_last_y: 0,
            mouse_last_b: 0,
            mouse_drag_flag: 0,
            mouse_scrolling_flag: 0,
            mouse_slider_mpos: 0,
            mouse_last_pane: 0,
            mouse_drag_update: None,
            mouse_drag_release: None,
            key_timer: TimerHandle::default(),
            key_tree: None,
        }
    }
}
#[repr(C)]
pub struct tty_term {
    pub name: Option<::std::ffi::CString>,
    pub features: ::core::ffi::c_int,
    pub acs: [[::core::ffi::c_char; 2]; 256],
    pub codes: Box<[TtyCode]>,
    pub flags: ::core::ffi::c_int,
}
pub type winlinks = ::std::collections::BTreeMap<::core::ffi::c_int, ::std::boxed::Box<winlink>>;
/// The modes a pane has open, the one it is showing first. A mode belongs to
/// the pane until `window_pane_reset_mode` takes it off.
pub type window_modes = ::std::vec::Vec<::std::boxed::Box<window_mode_entry>>;
/// The links to one window, in the order they were made.
/// The sessions' links to a window, in the order they were made, named by
/// the session that holds each and the index it holds it at. A link belongs
/// to its session, not to the window.
pub(crate) type window_winlinks = ::std::vec::Vec<(SessionWeak, ::core::ffi::c_int)>;

/// A session's most-recently-used window links, most recent first.
/// The indexes of the links on a session's most-recent order, newest first.
pub type winlink_stack = ::std::vec::Vec<::core::ffi::c_int>;
/// The panes a window holds, in pane-index order, and the window owns them:
/// a pane lives exactly as long as its place in this vector. Each pane sits
/// behind its own `Box` so the raw `*mut window_pane` the layout cells, the
/// find state and the pane registry keep stays put while the window owns it.
/// A plain Rust vector rather than a TAILQ so that dropping the window tears
/// it down without writing through per-pane back-pointers.
pub type window_panes_t = ::std::vec::Vec<::std::boxed::Box<window_pane>>;
/// An ordered pile of panes — the most-recently-used stack, most recent
/// first, and the stacking order, topmost first. A plain Rust vector rather
/// than a TAILQ so that dropping the window tears it down without writing
/// through per-pane back-pointers.
/// The ids of the panes on one of a window's two orders, most recent or
/// topmost first. A pane is named by its id, so an order never holds one the
/// window has given up.
pub type window_pane_stack_t = ::std::vec::Vec<u_int>;
/// The cells directly under one layout cell, left to right or top to bottom.
/// A cell belongs to the list it hangs in.
pub type layout_cells = ::std::vec::Vec<::std::boxed::Box<layout_cell>>;
pub(crate) type client_files_t = ::std::collections::BTreeMap<::core::ffi::c_int, ClientFileRef>;

/// Which set of files a file belongs to: one client's, the whole process's,
/// or none at all once it has been taken out of the set it was in.
#[derive(Clone, Default)]
pub(crate) enum FileOwner {
    #[default]
    None,
    /// The files of one client, observed rather than held.
    Client(ClientWeak),
    /// A set the caller keeps itself and outlives every file in it: the one
    /// set the client process holds for all of its files.
    Held(*mut client_files_t),
}

impl FileOwner {
    /// The set itself, or null once whatever held it has gone.
    pub(crate) fn tree(&self) -> *mut client_files_t {
        match self {
            FileOwner::None => ::core::ptr::null_mut(),
            FileOwner::Client(watched) => watched
                .upgrade()
                .map_or(::core::ptr::null_mut(), |c| unsafe {
                    &raw mut (*c.as_ptr()).files
                }),
            FileOwner::Held(files) => *files,
        }
    }
}
pub type client_windows = ::std::collections::BTreeMap<u_int, client_window>;
pub type client_file_cb = Option<
    unsafe fn(
        *mut client,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        *mut Buf,
        ClientFileData,
    ) -> (),
>;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum OverlayData {
    None,
    Menu(*mut menu_data),
    Popup(*mut popup_data),
    DisplayPanes(*mut cmd_display_panes_data),
}

#[derive(Default)]
#[repr(C)]
pub enum OverlayState {
    #[default]
    None,
    Menu(Box<menu_data>),
    Popup(PopupDataRef),
    DisplayPanes(Box<cmd_display_panes_data>),
}

impl OverlayState {
    pub fn data(&self) -> OverlayData {
        match self {
            OverlayState::None => OverlayData::None,
            OverlayState::Menu(data) => {
                OverlayData::Menu(data.as_ref() as *const menu_data as *mut menu_data)
            }
            OverlayState::Popup(data) => OverlayData::Popup(data.as_ptr()),
            OverlayState::DisplayPanes(data) => OverlayData::DisplayPanes(data.as_ref()
                as *const cmd_display_panes_data
                as *mut cmd_display_panes_data),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, OverlayState::None)
    }

    pub fn menu(&self) -> *mut menu_data {
        self.data().menu()
    }

    pub fn popup(&self) -> *mut popup_data {
        self.data().popup()
    }

    pub fn display_panes(&self) -> *mut cmd_display_panes_data {
        self.data().display_panes()
    }
}

impl client {
    /// The key table the client's next key is looked up in, or null before
    /// one has been given to it.
    pub(crate) fn keytable(&self) -> *mut key_table {
        self.keytable_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), KeyTableRef::as_ptr)
    }

    pub(crate) fn current_overlay_data(&self) -> OverlayData {
        self.overlay_data_view
            .unwrap_or_else(|| self.overlay_data.data())
    }

    /// The overlay the client is showing, which is `Overlay::None` when it is
    /// showing none.
    pub(crate) fn overlay(&self) -> Overlay {
        self.overlay
    }

    /// Which overlay answers for the region the current drawing may cover.
    pub(crate) fn overlay_check(&self) -> OverlayCheck {
        self.overlay_check
    }

    /// What the overlay is showing, which the module that put it up owns.
    pub(crate) fn overlay_data(&self) -> &OverlayState {
        &self.overlay_data
    }

    /// Puts `overlay` up, showing `data`. The region goes to whichever
    /// overlay answers for it, and any retargeting the last one left behind
    /// is dropped.
    pub(crate) fn set_overlay(&mut self, overlay: Overlay, data: OverlayState) {
        self.overlay_check = overlay.check();
        self.overlay = overlay;
        self.overlay_data = data;
        self.overlay_data_view = None;
    }

    /// Takes the overlay down and hands back what it was showing, so that the
    /// caller can free it. Nothing answers for the region afterwards.
    pub(crate) fn take_overlay(&mut self) -> (Overlay, OverlayState) {
        let overlay = ::core::mem::replace(&mut self.overlay, Overlay::None);
        let data = ::core::mem::take(&mut self.overlay_data);
        self.overlay_check = OverlayCheck::None;
        self.overlay_data_view = None;
        (overlay, data)
    }

    /// Points the region at `view` for the drawing that follows, which is how
    /// a popup hands it to the menu it carries and takes it back afterwards.
    pub(crate) fn set_overlay_view(&mut self, view: OverlayView) {
        match view {
            OverlayView::Menu(md) => {
                self.overlay_check = OverlayCheck::Menu;
                self.overlay_data_view = Some(OverlayData::Menu(md));
            }
            OverlayView::Nothing => {
                self.overlay_check = OverlayCheck::None;
                self.overlay_data_view = Some(OverlayData::None);
            }
            OverlayView::Popup => {
                self.overlay_check = OverlayCheck::Popup;
                self.overlay_data_view = None;
            }
        }
    }

    /// Gives the drawing data back to the overlay itself, which is what a
    /// popup does once the menu it was carrying is gone.
    pub(crate) fn clear_overlay_view(&mut self) {
        self.overlay_data_view = None;
    }
}

/// Who answers for the region the current drawing may cover, while a popup
/// points it somewhere other than itself.
#[derive(Copy, Clone)]
pub(crate) enum OverlayView {
    /// The menu the popup carries, which owns the drawing data as well.
    Menu(*mut menu_data),
    /// Nobody, so no drawing may be covered and there is nothing to draw
    /// with, which is the state a popup writes its own screen under.
    Nothing,
    /// The popup itself, drawing with what the overlay was put up with.
    Popup,
}
/// Which overlay a client is showing, and so which module owns
/// `client::overlay_data`.
///
/// A client carries one overlay at a time, installed by
/// `server_client_set_overlay` and taken down by
/// `server_client_clear_overlay`; the three that exist are named here rather
/// than reached through a table of callbacks, so that asking which one is up
/// — as `popup_present` does — is a comparison of values.
///
/// `None` is the zero byte, because a client arrives from `xcalloc`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Overlay {
    #[default]
    None = 0,
    Menu,
    Popup,
    /// `display-panes`, which reads keys unless `-N` was given.
    DisplayPanes {
        keys: bool,
    },
}

/// Which overlay decides what the current drawing may cover.
///
/// This follows the overlay for as long as one is up, but a popup retargets
/// it while it draws or feeds its own pane: the menu it carries answers for
/// the region then, and nothing answers while the popup writes its own
/// screen.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum OverlayCheck {
    #[default]
    None = 0,
    Menu,
    Popup,
}

impl Overlay {
    pub fn is_some(self) -> bool {
        self != Overlay::None
    }
    pub fn is_none(self) -> bool {
        self == Overlay::None
    }
}

impl OverlayCheck {
    pub fn is_some(self) -> bool {
        self != OverlayCheck::None
    }
    pub fn is_none(self) -> bool {
        self == OverlayCheck::None
    }
}
/// Who put the prompt on the status line, and so who reads what is typed
/// into it and owns `client::prompt_data`.
///
/// A client shows one prompt at a time, opened by `status_prompt_set`. The
/// variants are the callers that open one: naming them keeps the answer to
/// "is this still my prompt?" — which `cmd_command_prompt` asks before it
/// continues its command queue — a comparison of values.
///
/// A client keeps the last value after `status_prompt_clear`, exactly as it
/// kept the last callback pointer; `client::prompt_string` says whether a
/// prompt is up.
///
/// `None` is the zero byte, because a client arrives from `xcalloc`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Prompt {
    #[default]
    None = 0,
    CommandPrompt,
    ConfirmBefore,
    ModeTreeSearch,
    ModeTreeFilter,
    WindowTreeCommand,
    WindowTreeKillCurrent,
    WindowTreeKillTagged,
    CustomizeSetOption,
    CustomizeSetCommand,
    CustomizeSetNote,
    CustomizeChangeCurrent,
    CustomizeChangeTagged,
    /// A prompt opened by a unit test, which records what it is answered.
    #[cfg(test)]
    Recorder,
}

impl Prompt {
    pub fn is_some(self) -> bool {
        self != Prompt::None
    }
    pub fn is_none(self) -> bool {
        self == Prompt::None
    }
}

/// The state owned by the prompt currently displayed on a client.
///
/// Prompt-private records are boxed directly. Mode prompts instead box a weak
/// handle to their shared mode state, so closing the mode drops that state and
/// the prompt's later input finds nothing to act on. A customize prompt boxes
/// its own item, which carries that weak handle in `prompt_owner`.
#[derive(Default)]
#[repr(C)]
pub enum PromptData {
    #[default]
    None,
    CommandPrompt(Box<cmd_command_prompt_cdata>),
    ConfirmBefore(Box<cmd_confirm_before_data>),
    ModeTree(Box<ModeTreeDataWeak>),
    WindowTree(Box<WindowTreeModeDataWeak>),
    CustomizeSet(Box<window_customize_itemdata>),
    CustomizeChange(Box<WindowCustomizeModeDataWeak>),
}

impl PartialEq for PromptData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::CommandPrompt(left), Self::CommandPrompt(right)) => {
                ::core::ptr::eq(&**left, &**right)
            }
            (Self::ConfirmBefore(left), Self::ConfirmBefore(right)) => {
                ::core::ptr::eq(&**left, &**right)
            }
            (Self::ModeTree(left), Self::ModeTree(right)) => left.ptr_eq(right),
            (Self::WindowTree(left), Self::WindowTree(right)) => left.ptr_eq(right),
            (Self::CustomizeSet(left), Self::CustomizeSet(right)) => {
                ::core::ptr::eq(&**left, &**right)
            }
            (Self::CustomizeChange(left), Self::CustomizeChange(right)) => left.ptr_eq(right),
            _ => false,
        }
    }
}

impl Eq for PromptData {}

impl ::core::fmt::Debug for PromptData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(match self {
            Self::None => "None",
            Self::CommandPrompt(_) => "CommandPrompt(..)",
            Self::ConfirmBefore(_) => "ConfirmBefore(..)",
            Self::ModeTree(_) => "ModeTree(..)",
            Self::WindowTree(_) => "WindowTree(..)",
            Self::CustomizeSet(_) => "CustomizeSet(..)",
            Self::CustomizeChange(_) => "CustomizeChange(..)",
        })
    }
}

#[derive(Default)]
#[repr(C)]
pub enum JobData {
    #[default]
    None,
    Format(*mut format_job),
    Popup(PopupDataWeak),
    RunShell(Box<cmd_run_shell_data>),
    IfShell(Box<cmd_if_shell_data>),
}

/// Every client the server holds, in the order they connected. A client
/// belongs to the client registry, not to this list.
pub(crate) type clients_t = ::std::vec::Vec<ClientRef>;
pub(crate) type sessions_t = ::std::collections::BTreeMap<::std::ffi::CString, SessionRef>;
/// Where a command being parsed came from. The default names no source file,
/// no issuing item or client, and no target.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_parse_input {
    pub flags: ::core::ffi::c_int,
    /// The file the command line was read from, if it was read from one.
    pub file: Option<::std::ffi::CString>,
    pub line: u_int,
    pub item: *mut cmdq_item,
    /// The client the parse is for, observed rather than held.
    pub(crate) c: Option<ClientWeak>,
    pub fs: cmd_find_state,
}

impl cmd_parse_input {
    /// The file the command line came from, or nothing when it came from
    /// none.
    pub fn file(&self) -> Option<&::core::ffi::CStr> {
        self.file.as_deref()
    }

    /// The client the parse is for, or null when it is for none.
    pub fn client(&self) -> *mut client {
        self.c
            .as_ref()
            .and_then(ClientWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |c| c.as_ptr())
    }
}
/// What one spawn was asked for. The strings are borrowed from the command
/// that asked; the objects are views into the graph the spawn itself changes,
/// so they stay raw.
#[derive(Default)]
#[repr(C)]
pub struct spawn_context<'a> {
    pub item: Option<crate::cmd::CmdqItemWeak>,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub tc: Option<ClientWeak>,
    pub wp0: *mut window_pane,
    pub lc: *mut layout_cell,
    pub name: Option<&'a ::core::ffi::CStr>,
    pub argv: Vec<::std::ffi::CString>,
    pub environ: Option<Box<environ_t>>,
    pub idx: ::core::ffi::c_int,
    pub cwd: Option<&'a ::core::ffi::CStr>,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub enum TtyCtxArg {
    #[default]
    None,
    Pane(*mut window_pane),
    Popup(*mut popup_data),
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct tty_ctx {
    pub s: *mut screen,
    pub redraw_cb: tty_ctx_redraw_cb,
    pub set_client_cb: tty_ctx_set_client_cb,
    pub arg: TtyCtxArg,
    pub cell: *const grid_cell,
    pub flags: ::core::ffi::c_int,
    pub value: TtyCtxValue,
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
    pub palette: *mut colour_palette,
    pub wox: u_int,
    pub woy: u_int,
    pub wsx: u_int,
    pub wsy: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_ctx_data {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_redraw_cb = Option<unsafe fn(&tty_ctx) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_ctx_sel {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb = Option<unsafe fn(&mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
/// What a terminal command carries besides its position: a count, a run of
/// bytes, or a selection to hand to the terminal.
#[derive(Copy, Clone, Default)]
pub enum TtyCtxValue {
    #[default]
    None,
    Num(u_int),
    Data(tty_ctx_data),
    Sel(tty_ctx_sel),
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub arg: *mut popup_data,
    pub item: crate::screen::CItem,
    pub scrolled: u_int,
    pub bg: u_int,
}
pub type screen_write_init_ctx_cb = Option<unsafe fn(&mut screen_write_ctx, &mut tty_ctx) -> ()>;
/// An argument's kind and payload.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub enum ArgsValue {
    #[default]
    None,
    String(::std::ffi::CString),
    Commands {
        cmdlist: Option<CmdListRef>,
        cached: Option<::std::ffi::CString>,
    },
}

impl ArgsValue {
    /// The string the value holds, which every caller of this has already
    /// established it is.
    pub fn string(&self) -> &::core::ffi::CStr {
        match self {
            ArgsValue::String(string) => string,
            _ => panic!("not a string argument"),
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct args_value_t {
    pub value: ArgsValue,
}
pub type args_values_t = ::std::vec::Vec<::std::boxed::Box<args_value_t>>;
#[derive(Default)]
#[repr(C)]
pub struct ibuf {
    pub buf: ::bytes::BytesMut,
    pub size: size_t,
    pub max: size_t,
    pub wpos: size_t,
    pub rpos: size_t,
    pub fd: ::core::ffi::c_int,
    /// Whether the bytes were copied out of somebody else's range rather
    /// than allocated here. A borrowed buffer may be read, but not grown,
    /// resized, given a descriptor, or handed to a queue.
    pub borrowed: bool,
}
#[derive(Default)]
#[repr(C)]
pub struct imsg {
    pub hdr: imsg_hdr,
    /// The message body, a view into the buffer below.
    pub data: *mut ::core::ffi::c_uchar,
    /// The buffer the message was read out of, which the message owns until
    /// it is given up or handed on to a queue.
    pub buf: Option<Box<ibuf>>,
}
pub struct message_entry {
    pub msg: ::std::ffi::CString,
    pub msg_num: u_int,
    pub msg_time: timeval,
}
#[repr(C)]
pub struct hyperlinks_uri {
    pub inner: u_int,
    pub internal_id: Option<::std::ffi::CString>,
    pub external_id: Option<::std::ffi::CString>,
    pub uri: Option<::std::ffi::CString>,
}
#[repr(C)]
pub struct args_entry {
    pub flag: u_char,
    pub values: args_values_t,
    pub count: u_int,
    pub flags: ::core::ffi::c_int,
}
pub type args_tree = ::std::collections::BTreeMap<u_char, ::std::boxed::Box<args_entry>>;
/// The children of one tree item, or the tree's own top level, in the order
/// they were added. An item belongs to the list it sits on.
pub type mode_tree_list = ::std::vec::Vec<::std::boxed::Box<mode_tree_item>>;

pub fn cstr_ptr(s: &Option<::std::ffi::CString>) -> *mut ::core::ffi::c_char {
    match s {
        Some(s) => s.as_ptr() as *mut ::core::ffi::c_char,
        None => ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }
}

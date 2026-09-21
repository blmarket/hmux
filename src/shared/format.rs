//! Authoritative format declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::{cmd, cmdq_item};
use super::job::job;
use super::mouse::mouse_event;
use super::pane::window_pane;
use super::paste::paste_buffer;
use super::session::session;
use super::window::{window, winlink};
pub const FORMAT_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;

pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;

pub const FORMAT_STATUS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const FORMAT_FORCE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;

pub const FORMAT_PANE: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;

pub const FORMAT_WINDOW: ::core::ffi::c_uint = 0x40000000 as ::core::ffi::c_uint;

pub const FORMAT_MAX_WIDTH: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;

pub const FORMAT_MAX_REPEAT: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;

pub const FORMAT_MAX_PRECISION: ::core::ffi::c_int = 100 as ::core::ffi::c_int;

pub const FORMAT_TIMESTRING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const FORMAT_BASENAME: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;

pub const FORMAT_DIRNAME: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;

pub const FORMAT_QUOTE_SHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;

pub const FORMAT_LITERAL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

pub const FORMAT_EXPAND: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;

pub const FORMAT_EXPANDTIME: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;

pub const FORMAT_SESSIONS: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;

pub const FORMAT_WINDOWS: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;

pub const FORMAT_PANES: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;

pub const FORMAT_PRETTY: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;

pub const FORMAT_LENGTH: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;

pub const FORMAT_WIDTH: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;

pub const FORMAT_QUOTE_STYLE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;

pub const FORMAT_WINDOW_NAME: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;

pub const FORMAT_SESSION_NAME: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;

pub const FORMAT_CHARACTER: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;

pub const FORMAT_COLOUR: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;

pub const FORMAT_CLIENTS: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;

pub const FORMAT_NOT: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;

pub const FORMAT_NOT_NOT: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;

pub const FORMAT_REPEAT: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;

pub const FORMAT_QUOTE_ARGUMENTS: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;

pub const FORMAT_RELATIVE: ::core::ffi::c_int = 0x800000 as ::core::ffi::c_int;

pub const FORMAT_CLIENT_TERMCAP: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;

pub const FORMAT_CLIENT_TERMFEAT: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;

pub const FORMAT_CLIENT_ENVIRON: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;

pub const FORMAT_COLOUR_ESC_FG: ::core::ffi::c_int = 0x8000000 as ::core::ffi::c_int;

pub const FORMAT_COLOUR_ESC_BG: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;

pub const FORMAT_QUOTE_SHELL_SQ: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;

pub const FORMAT_OPTIONS: ::core::ffi::c_int = 0x40000000 as ::core::ffi::c_int;

pub const FORMAT_ENVIRON: ::core::ffi::c_ulonglong = 0x80000000 as ::core::ffi::c_ulonglong;

pub const FORMAT_DIFFERENCE: ::core::ffi::c_ulonglong = 0x100000000 as ::core::ffi::c_ulonglong;

pub const FORMAT_CYCLE: ::core::ffi::c_ulonglong = 0x200000000 as ::core::ffi::c_ulonglong;

pub const FORMAT_LOOP_LIMIT: ::core::ffi::c_int = 100 as ::core::ffi::c_int;

pub const FORMAT_TIME_LIMIT: ::core::ffi::c_int = 100 as ::core::ffi::c_int;

pub const FORMAT_TIME_LOOP_CHECK: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;

pub const FORMAT_CYCLE_PERIOD: ::core::ffi::c_int = 100 as ::core::ffi::c_int;

pub const FORMAT_EXPAND_TIME: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const FORMAT_EXPAND_NOJOBS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;

pub const FORMAT_EXPAND_NOCYCLE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_tree {
    pub type_0: format_type,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub w: *mut window,
    pub wp: *mut window_pane,
    pub pb: *mut paste_buffer,
    pub item: *mut cmdq_item,
    pub client: *mut client,
    pub flags: ::core::ffi::c_int,
    pub tag: u_int,
    pub m: mouse_event,
    pub tree: format_entry_tree,
}

// Jobs remain separately C-allocated because process callbacks retain their addresses.
// Keys own the original command bytes, ordered exactly like tag followed by strcmp.
#[derive(Default)]
pub struct format_job_tree {
    pub(crate) entries: std::collections::BTreeMap<(u_int, Vec<u8>), *mut format_job>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_entry_tree {
    pub rbh_root: *mut format_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_entry {
    pub key: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub time: time_t,
    pub cb: format_cb,
    pub entry: format_entry_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_entry_entry {
    pub rbe_left: *mut format_entry,
    pub rbe_right: *mut format_entry,
    pub rbe_parent: *mut format_entry,
    pub rbe_color: ::core::ffi::c_int,
}

pub type format_cb = Option<unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void>;

pub type format_type = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_job {
    pub client: *mut client,
    pub tag: u_int,
    pub cmd: *const ::core::ffi::c_char,
    pub expanded: *const ::core::ffi::c_char,
    pub last: time_t,
    pub out: *mut ::core::ffi::c_char,
    pub updated: ::core::ffi::c_int,
    pub job: *mut job,
    pub status: ::core::ffi::c_int,
}

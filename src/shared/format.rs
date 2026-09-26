//! Authoritative format declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::cmdq_item;
use super::job::job;
use super::mouse::mouse_event;
use super::pane::window_pane;
use super::paste::paste_buffer;
use super::session::session;
use super::window::{window, winlink};
use std::collections::BTreeMap;
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

/// Box-owned by format_create; borrowed pointers are invalid after format_free.
#[derive(Copy, Clone, Default)]
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

/// Scoped access to a format tree while a lazy value callback is running.
/// The context owns no tree data and is valid only for that callback call.
pub struct FormatContext {
    tree: std::ptr::NonNull<format_tree>,
}

impl FormatContext {
    pub(crate) fn from_tree(tree: std::ptr::NonNull<format_tree>) -> Self {
        Self { tree }
    }

    pub fn pane(&self) -> Option<std::ptr::NonNull<window_pane>> {
        // The callback dispatcher guarantees that this handle refers to a live tree.
        unsafe { std::ptr::NonNull::new(self.tree.as_ref().wp) }
    }

    pub(crate) fn tree(&self) -> std::ptr::NonNull<format_tree> {
        self.tree
    }
}

// Jobs live in stable Rust allocations because process callbacks retain their addresses.
// Keys own the original command bytes, ordered exactly like tag followed by strcmp.
#[derive(Default)]
pub struct format_job_tree {
    pub(crate) entries: std::collections::BTreeMap<(u_int, Vec<u8>), *mut format_job>,
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct format_entry_tree {
    pub entries: *mut format_entry_tree_storage,
}

/// Rust-owned ordering storage for a format tree's C-allocated entries.
/// Keys retain the original C string bytes so ordering matches `strcmp`.
#[derive(Default)]
pub struct format_entry_tree_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut format_entry>,
}

pub struct format_entry {
    pub key: std::ffi::CString,
    pub value: Option<std::ffi::CString>,
    pub time: time_t,
    pub(crate) owned_cb: Option<Box<dyn FnMut(&mut FormatContext) -> Option<std::ffi::CString>>>,
}

impl format_entry {
    pub fn empty() -> Self {
        Self {
            key: Default::default(),
            value: Default::default(),
            time: Default::default(),
            owned_cb: Default::default(),
        }
    }
}

pub type format_type = ::core::ffi::c_uint;

#[repr(C)]
pub struct format_job {
    pub client: *mut client,
    pub tag: u_int,
    pub cmd: std::ffi::CString,
    pub expanded: Option<std::ffi::CString>,
    pub last: time_t,
    pub out: Option<std::ffi::CString>,
    pub updated: ::core::ffi::c_int,
    pub job: *mut job,
    pub status: ::core::ffi::c_int,
}

impl format_job {
    pub fn empty() -> Self {
        Self {
            client: Default::default(),
            tag: Default::default(),
            cmd: Default::default(),
            expanded: Default::default(),
            last: Default::default(),
            out: Default::default(),
            updated: Default::default(),
            job: Default::default(),
            status: Default::default(),
        }
    }
}

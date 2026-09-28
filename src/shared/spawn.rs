//! Authoritative spawn declarations, shared by the C translation units.

use super::abi::pid_t;
use super::client::client;
use super::command::cmdq_item;
use super::environment::environ;
use super::layout::layout_cell;
use super::pane::window_pane;
use super::session::session;
use super::window::winlink;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_HORIZONTAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const SPAWN_KILL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_RESPAWN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const SPAWN_SPLIT: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const SPAWN_MODAL: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const SPAWN_FLOATOVERZOOM: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const SPAWN_NONOTIFY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

pub struct spawn_editor_state {
    pub id: EditorId,
    pub path: std::ffi::CString,
    pub pid: pid_t,
    pub cb: spawn_finish_edit_cb,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorId(pub(crate) u64);

/// Observe one pane-owned editor without keeping its Box alive. The identity
/// prevents a delayed completion from clearing a replacement editor.
pub struct EditorHandle {
    pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    id: EditorId,
}

impl EditorHandle {
    /// The editor pointer must be live and installed in this pane.
    pub unsafe fn new(
        pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
        editor: std::ptr::NonNull<spawn_editor_state>,
    ) -> Self {
        assert_eq!(
            (*pane.get()).editor.as_ref().map(|state| state.id),
            Some(editor.as_ref().id),
            "editor must belong to the pane",
        );
        Self {
            pane: std::rc::Rc::downgrade(pane),
            id: editor.as_ref().id,
        }
    }

    pub fn matches(&self, editor: &spawn_editor_state) -> bool {
        editor.id == self.id
    }

    pub fn cancel(&self) {
        let Some(pane) = self.pane.upgrade() else { return };
        let pane = unsafe { &mut *pane.get() };
        if let Some(editor) = pane.editor.as_mut().filter(|editor| editor.id == self.id) {
            editor.cb = None;
        }
    }

    pub fn pid(&self) -> pid_t {
        let Some(pane) = self.pane.upgrade() else { return -1 };
        let pane = unsafe { &*pane.get() };
        pane.editor
            .as_ref()
            .filter(|editor| editor.id == self.id)
            .map_or(-1, |editor| editor.pid)
    }
}

/// Rust-only callback so the editor result can move as owned binary bytes.
pub type spawn_finish_edit_cb =
    Option<Box<dyn FnOnce(std::ptr::NonNull<spawn_editor_state>, Option<Vec<u8>>)>>;

#[repr(C)]
pub struct spawn_context {
    pub item: *mut cmdq_item,
    pub s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>>,
    pub wl: refbox::Weak<winlink>,
    /// Retain the target client for the duration of this spawn operation.
    pub tc: Option<std::rc::Rc<std::cell::UnsafeCell<client>>>,
    pub wp0: Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    pub lc: *mut layout_cell,
    pub name: Option<std::ffi::CString>,
    pub argv: Vec<std::ffi::CString>,
    pub environ: Option<Box<environ>>,
    pub idx: ::core::ffi::c_int,
    pub cwd: Option<std::ffi::CString>,
    pub flags: ::core::ffi::c_int,
}

impl spawn_context {
    /// A borrowed view for the synchronous legacy spawn calls.
    pub fn wl_ptr(&self) -> *mut winlink {
        if self.wl.is_alive() {
            self.wl.as_ptr().cast_mut()
        } else {
            std::ptr::null_mut()
        }
    }

    /// The caller supplies a live winlink owned by a session index.
    pub unsafe fn set_wl(&mut self, wl: *mut winlink) {
        self.wl = wl.as_ref().map_or_else(refbox::Weak::new, |wl| wl.observer.clone());
    }
}

//! Authoritative window declarations, shared by the C translation units.

use super::abi::{timeval, u_int, uint64_t};
use super::arguments::args;
use super::client::client;
use super::command::{cmd_find_state, cmdq_item};
use super::event::event;
use super::format::format_tree;
use super::grid::grid_cell;
use super::key::key_code;
use super::layout::layout_cell;
use super::menu::MenuOwner;
use super::mouse::mouse_event;
use super::options::options;
use super::pane::{window_pane, window_pane_history, window_panes, PANE_MINIMUM};
use super::screen::screen;
use super::session::session;

pub const WINDOW_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINDOW_ALERTFLAGS: ::core::ffi::c_int = WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINDOW_MINIMUM: ::core::ffi::c_int = PANE_MINIMUM;
pub const WINDOW_MAXIMUM: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const WINDOW_SIZE_LARGEST: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WINDOW_SIZE_MANUAL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const WINDOW_PANE_NO_MODE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WINDOW_SIZE_LATEST: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const WINDOW_RESIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const WINLINK_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINDOW_MODE_HIDE_PANE_STATUS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_MODE_NO_STACK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_MODE_HIDE_SCROLLBARS: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;

#[repr(C)]
pub struct winlinks {
    /// The session owns the index allocation. The index itself owns the
    /// `RefBox` allocations for winlinks created by `winlink_add`.
    pub storage: Option<
        refbox::RefBox<std::collections::BTreeMap<::core::ffi::c_int, refbox::RefBox<winlink>>>,
    >,
}

#[derive(Default)]
#[repr(C)]
pub struct winlink {
    /// Observe this allocation independently of its current index/key.
    pub(crate) observer: refbox::Weak<winlink>,
    pub idx: ::core::ffi::c_int,
    pub session: std::rc::Weak<std::cell::UnsafeCell<session>>,
    pub window_owner: Option<WindowOwner>,
    pub flags: ::core::ffi::c_int,
    pub entry: winlink_entry,
}

#[derive(Default)]
#[repr(C)]
pub struct winlink_entry {
    pub owner: refbox::Weak<std::collections::BTreeMap<::core::ffi::c_int, refbox::RefBox<winlink>>>,
}

#[derive(Default)]
#[repr(C)]
/// Rc-owned window record; retain/release preserves pre-close notifications.
pub struct window {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: std::rc::Weak<std::cell::UnsafeCell<window>>,
    pub id: u_int,
    /// Nonowning identity of the client last active in this window.
    pub latest: std::rc::Weak<std::cell::UnsafeCell<super::client::client>>,
    pub name: std::ffi::CString,
    pub name_event: event,
    pub name_time: timeval,
    pub alerts_timer: event,
    pub offset_timer: event,
    pub activity_time: timeval,
    pub creation_time: timeval,
    /// Current pane identity; the pane index owns the allocation.
    pub active: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Current modal pane is observed; the pane index owns its lifetime.
    pub modal: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Pane to restore after modal dismissal; does not own that pane.
    pub modal_last: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Saved zoom target observes its pane without extending its lifetime.
    pub was_zoomed: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub last_panes: window_pane_history,
    pub z_index: window_panes,
    pub panes: window_panes,
    pub lastlayout: ::core::ffi::c_int,
    pub layout_root: Option<Box<layout_cell>>,
    pub saved_layout_root: Option<Box<layout_cell>>,
    pub old_layout: Option<std::ffi::CString>,
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
    pub redraw_scene_generation: uint64_t,
    pub menu: Option<MenuOwner>,
    pub menu_last_px: u_int,
    pub menu_last_py: u_int,
    pub last_new_pane_x: u_int,
    pub last_new_pane_y: u_int,
    pub sb: ::core::ffi::c_int,
    pub sb_pos: ::core::ffi::c_int,
    pub inside_cell: grid_cell,
    pub outside_cell: grid_cell,
    pub flags: ::core::ffi::c_int,
    pub alerts_queued: ::core::ffi::c_int,
    pub options: Option<Box<options>>,
    pub winlinks: window_winlinks,
    pub entry: window_entry,
}

/// The global index observes windows; winlinks and callbacks own them.
pub type WindowIndex = std::collections::BTreeMap<
    u_int,
    std::rc::Weak<std::cell::UnsafeCell<window>>,
>;

#[derive(Default)]
#[repr(C)]
pub struct window_entry {
    /// Weak traversal handle into the index; cleared when this window is removed.
    pub owner: refbox::Weak<WindowIndex>,
}

/// Address identity only: never used to recover or dereference a model pointer.
/// The ordered weak handles keep their control blocks allocated, preventing
/// address reuse until the corresponding identity is removed from the index.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WinlinkIdentity(usize);

impl WinlinkIdentity {
    pub(crate) fn of(link: *const winlink) -> Self {
        Self(link.addr())
    }
}

#[derive(Default)]
pub struct WindowWinlinksStorage {
    pub(crate) ordered: Vec<refbox::Weak<winlink>>,
    pub(crate) positions: std::collections::HashMap<WinlinkIdentity, usize>,
}

#[derive(Default)]
#[repr(C)]
pub struct window_winlinks {
    /// Ordered non-owning handles; session BTreeMaps own the RefBox allocations.
    pub storage: Option<Box<WindowWinlinksStorage>>,
}

#[repr(C)]
pub struct window_mode_entry {
    /// The pane owns this entry; retaining it here would create a cycle.
    pub wp: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    /// Non-owning source; copy mode keeps an independent screen snapshot.
    pub swp: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub mode: &'static window_mode,
    /// Owns mode payloads that need a stable address but no shared ownership.
    pub boxed_data: Option<Box<dyn std::any::Any>>,
    /// Owns payloads shared with callbacks that can outlive their mode entry.
    pub data_owner: Option<std::rc::Rc<dyn std::any::Any>>,
    pub prefix: u_int,
    pub kill: ::core::ffi::c_int,
}

impl window_mode_entry {
    pub fn boxed_data_ptr<T: std::any::Any>(&self) -> Option<*mut T> {
        self.boxed_data
            .as_ref()?
            .downcast_ref::<std::cell::UnsafeCell<T>>()
            .map(std::cell::UnsafeCell::get)
    }

    pub fn shared_data_ptr<T: std::any::Any>(&self) -> Option<*mut T> {
        self.data_owner
            .as_ref()?
            .downcast_ref::<std::cell::UnsafeCell<T>>()
            .map(std::cell::UnsafeCell::get)
    }

    /// Retain an Rc-backed payload across callbacks that may remove this entry.
    pub fn retained_data<T: std::any::Any>(&self) -> Option<std::rc::Rc<T>> {
        self.data_owner.as_ref().map(|owner| {
            owner.clone().downcast::<T>().expect("mode payload type")
        })
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode {
    pub name: &'static ::std::ffi::CStr,
    pub default_format: Option<&'static std::ffi::CStr>,
    pub flags: ::core::ffi::c_int,
    pub init: Option<
        unsafe fn(
            *mut window_mode_entry,
            *mut cmdq_item,
            *mut cmd_find_state,
            *mut args,
        ) -> *mut screen,
    >,
    pub free: Option<unsafe fn(*mut window_mode_entry) -> ()>,
    pub resize: Option<unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()>,
    pub update: Option<unsafe fn(*mut window_mode_entry) -> ()>,
    pub style_changed: Option<unsafe fn(*mut window_mode_entry) -> ()>,
    pub key: Option<
        unsafe fn(
            *mut window_mode_entry,
            &std::rc::Rc<std::cell::UnsafeCell<client>>,
            *mut winlink,
            key_code,
            *mut mouse_event,
        ) -> (),
    >,
    pub key_table: Option<unsafe fn(*mut window_mode_entry) -> *const ::core::ffi::c_char>,
    pub command: Option<
        unsafe fn(
            *mut window_mode_entry,
            Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
            Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
            *mut winlink,
            *mut args,
            *mut mouse_event,
        ) -> (),
    >,
    pub formats: Option<unsafe fn(*mut window_mode_entry, *mut format_tree) -> ()>,
    pub get_screen: Option<unsafe fn(*mut window_mode_entry) -> *mut screen>,
    pub display_screen: Option<unsafe fn(*mut window_mode_entry) -> *mut screen>,
}

impl Default for window_mode {
    fn default() -> Self {
        Self {
            name: c"",
            default_format: None,
            flags: 0,
            init: None,
            free: None,
            resize: None,
            update: None,
            style_changed: None,
            key: None,
            key_table: None,
            command: None,
            formats: None,
            get_screen: None,
            display_screen: None,
        }
    }
}

#[repr(C)]
pub struct winlink_stack {
    /// Weak visit history; the session owns the deque and the ordered index
    /// owns each link.
    pub storage: Option<Box<std::collections::VecDeque<refbox::Weak<winlink>>>>,
}

#[repr(C)]
pub struct windows {
    /// The head owns the index; window records remain externally owned.
    pub storage: Option<refbox::RefBox<WindowIndex>>,
}

impl window {
    /// Legacy pointer view. The caller must keep the pane indexed or retain
    /// an Rc through its use; this liveness check is not a borrow guard.
    pub fn active_ptr(&self) -> *mut window_pane {
        self.active
            .upgrade()
            .map_or(std::ptr::null_mut(), |owner| owner.get())
    }

    /// The caller supplies a live Rc-backed pane.
    pub unsafe fn set_active(&mut self, pane: *mut window_pane) {
        self.active = pane.as_ref().map_or_else(std::rc::Weak::new, |pane| pane.observer.clone());
    }

    pub fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = Self::default();
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }

    pub fn layout_root_ptr(&mut self) -> Option<&mut layout_cell> {
        self.layout_root.as_deref_mut()
    }

    pub fn saved_layout_root_ptr(&mut self) -> Option<&mut layout_cell> {
        self.saved_layout_root.as_deref_mut()
    }
}

/// One existing window reference, including its last-close notification policy.
pub struct WindowOwner(pub(crate) Option<std::rc::Rc<std::cell::UnsafeCell<window>>>);

impl WindowOwner {
    /// Adopt an existing reference with the window's pre-release notification policy.
    pub fn adopt(owner: std::rc::Rc<std::cell::UnsafeCell<window>>) -> Self {
        Self(Some(owner))
    }

    /// # Safety
    /// The pointer must name a live Rc-owned window.
    pub unsafe fn retain(ptr: *mut window, from: &std::ffi::CStr) -> Self {
        Self(Some(crate::src::window::window_add_ref(ptr, from.as_ptr())))
    }

    pub fn as_ptr(&self) -> *mut window {
        super::rc::as_ptr(self.0.as_ref().expect("live window owner"))
    }

    pub fn as_rc(&self) -> &std::rc::Rc<std::cell::UnsafeCell<window>> {
        self.0.as_ref().expect("live window owner")
    }
}

impl Drop for WindowOwner {
    fn drop(&mut self) {
        if let Some(owner) = self.0.take() {
            unsafe {
                crate::src::window::window_remove_ref(
                    owner, c"WindowOwner::drop".as_ptr(),
                );
            }
        }
    }
}

impl winlink {
    pub fn window_ptr(&self) -> *mut window {
        self.window_owner.as_ref().map_or(std::ptr::null_mut(), WindowOwner::as_ptr)
    }
}

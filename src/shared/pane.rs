//! Authoritative pane declarations, shared by the C translation units.

use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ffi::CString;
use std::rc::{Rc, Weak};

use super::abi::{bitstr_t, pid_t, size_t, time_t, timeval, u_int, uint64_t};
use super::client::client;
use super::colour::{client_theme, colour_palette};
use super::command::cmdq_item;
use super::event::{bufferevent, event};
use super::grid::grid_cell;
use super::input::input_ctx;
use super::layout::layout_cell;
use super::options::options;
use super::prompt::{prompt_free_cb, prompt_type};
use super::screen::screen;
use super::spawn::spawn_editor_state;
use super::status::status_prompt_input_cb;
use super::style::{style, style_line_entry};
use super::window::{window, window_mode, window_mode_entry};
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct window_pane_offset {
    pub used: size_t,
}
#[repr(C)]
pub struct window_pane_resizes {
    /// The queue owner is optional so an empty queue has no heap allocation.
    pub storage: Option<Box<window_pane_resize_storage>>,
}

/// Each entry remains separately boxed so pointers used by cancellation keep
/// their address when the deque grows or other entries are removed.
pub type window_pane_resize_storage = VecDeque<Box<window_pane_resize>>;

impl Default for window_pane_resizes {
    fn default() -> Self {
        Self { storage: None }
    }
}

impl window_pane_resizes {
    pub fn as_ref(&self) -> Option<&window_pane_resize_storage> {
        self.storage.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_none_or(VecDeque::is_empty)
    }

    pub fn push_back(&mut self, resize: window_pane_resize) -> *mut window_pane_resize {
        let storage = self
            .storage
            .get_or_insert_with(|| Box::new(VecDeque::new()));
        let resize = Box::new(resize);
        let pointer = (&*resize) as *const window_pane_resize as *mut window_pane_resize;
        storage.push_back(resize);
        pointer
    }

    /// Retain only `except`; any removed entry pointer is invalid after return.
    pub fn clear_except(&mut self, except: *mut window_pane_resize) {
        if except.is_null() {
            self.storage = None;
            return;
        }
        let empty = {
            let Some(storage) = self.storage.as_mut() else {
                return;
            };
            storage.retain(|resize| {
                (&**resize) as *const window_pane_resize as *mut window_pane_resize == except
            });
            storage.is_empty()
        };
        if empty {
            self.storage = None;
        }
    }
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct window_pane_resize {
    pub sx: u_int,
    pub sy: u_int,
    pub osx: u_int,
    pub osy: u_int,
}
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_INPUTOFF: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_MAXIMUM: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_CLOSEONCLICK: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const PANE_CAPTUREALLKEYS: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCANCEL: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const PANE_ZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const PANE_CMDRUNNING: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_ALWAYS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_ACTIVITY: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const PANE_BORDER_COLOUR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_BORDER_ARROWS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_BORDER_BOTH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_NEWSTATUS: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PANE_DROP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_MODAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_AUTOHIDE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_FLOATOVERZOOM: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const PANE_EMPTY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_DEFAULT_PADDING: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_DEFAULT_WIDTH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_CHARACTER: ::core::ffi::c_int = ' ' as i32;
pub const PANE_FOCUSED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PANE_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_DESTROYED: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP_FLOATING: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM_FLOATING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;

/// The prompt cleanup closure owns this record; all other handles observe it.

pub struct window_pane_prompt {
    pub wp_id: u_int,
    pub c: Weak<UnsafeCell<client>>,
    pub inputcb: status_prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub type_0: prompt_type,
}

#[derive(Default)]
#[repr(C)]
pub struct window_pane {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub id: u_int,
    pub active_point: u_int,
    /// Nonowning parent; final window teardown provides a scoped fallback.
    pub window: Weak<UnsafeCell<window>>,
    pub options: Option<Box<options>>,
    pub layout_cell: *mut layout_cell,
    pub saved_layout_cell: *mut layout_cell,
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    /// Owns the bitmap for synchronized output; readers borrow its bytes.
    pub sync_dirty: Option<Box<[bitstr_t]>>,
    pub sync_dirty_size: u_int,
    pub sb_slider_y: u_int,
    pub sb_slider_h: u_int,
    pub sb_auto_visible: ::core::ffi::c_int,
    pub sb_auto_hover: ::core::ffi::c_int,
    pub sb_auto_timer: event,
    pub argv: Vec<std::ffi::CString>,
    pub shell: Option<CString>,
    pub cwd: Option<CString>,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub dead_time: timeval,
    pub wait_item: std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>,
    pub editor: Option<Box<spawn_editor_state>>,
    pub output_generation: uint64_t,
    pub last_output_time: time_t,
    pub last_prompt_time: time_t,
    pub cmd_start_time: time_t,
    pub cmd_end_time: time_t,
    pub cmd_status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    /// Observes the runtime-owned pane stream, including empty pane buffers.
    pub event: crate::src::reactor::StreamHandle,
    pub offset: window_pane_offset,
    pub base_offset: size_t,
    pub resize_queue: window_pane_resizes,
    pub resize_timer: event,
    pub sync_timer: event,
    pub ictx: Option<Box<input_ctx>>,
    pub cached_gc: grid_cell,
    pub cached_active_gc: grid_cell,
    pub cached_dim: u_int,
    pub cached_active_dim: u_int,
    pub palette: colour_palette,
    pub last_theme: client_theme,
    pub border_status_line: style_line_entry,
    pub pipe_fd: ::core::ffi::c_int,
    pub pipe_pid: pid_t,
    /// Observes the runtime-owned pipe stream; pipe_fd controls its lifetime.
    pub pipe_event: crate::src::reactor::StreamHandle,
    pub pipe_offset: window_pane_offset,
    pub screen_source: PaneScreenSource,
    pub base: screen,
    pub status_screen: screen,
    /// Pane-owned non-intrusive mode stack, drained before pane free.
    pub modes: window_pane_modes,
    pub searchstr: Option<CString>,
    pub searchregex: ::core::ffi::c_int,
    pub prompt: Option<refbox::RefBox<crate::src::shared::prompt::prompt>>,
    /// Callback-owned data; this observer cannot keep a closed prompt alive.
    pub prompt_data: refbox::Weak<crate::src::shared::pane::window_pane_prompt>,
    pub prompt_cx: u_int,
    pub border_gc_set: ::core::ffi::c_int,
    pub border_gc: grid_cell,
    pub active_border_gc_set: ::core::ffi::c_int,
    pub active_border_gc: grid_cell,
    pub control_bg: ::core::ffi::c_int,
    pub control_fg: ::core::ffi::c_int,
    pub scrollbar_style: style,
    pub tree_entry: window_pane_tree_entry,
}

impl window_pane {
    /// Retain the parent for the current operation. Release potentially final
    /// owners through window_remove_ref, as with other window handles.
    pub fn window_handle(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window>>> {
        self.window.upgrade()
    }

    /// Resolve the displayed screen while the pane and selected mode are live.
    pub unsafe fn screen_ptr(&self) -> *mut screen {
        match &self.screen_source {
            PaneScreenSource::Base => (&raw const self.base).cast_mut(),
            PaneScreenSource::Mode(observer) => {
                if !observer.is_alive() {
                    return std::ptr::null_mut();
                }
                let entry = observer.as_ptr().cast_mut();
                (*(*entry).mode).display_screen.expect("mode display screen getter")(entry)
            }
        }
    }

    pub fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        Self::empty().into_shared()
    }

    pub fn into_shared(mut self) -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new_cyclic(|observer| {
            self.observer = observer.clone();
            std::cell::UnsafeCell::new(self)
        })
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub enum PaneScreenSource {
    #[default]
    Base,
    Mode(refbox::Weak<window_mode_entry>),
}

#[derive(Default)]
#[repr(C)]
pub struct window_pane_tree_entry {
    pub owner: refbox::Weak<std::collections::BTreeMap<u_int, std::rc::Rc<std::cell::UnsafeCell<window_pane>>>>,
}

/// Ordered mode stack owned by a pane. Weak handles observe stable entries
/// while the stack moves and expire when an entry is removed and freed.
#[derive(Default)]
pub struct WindowPaneModesStorage {
    pub(crate) entries: Vec<refbox::RefBox<window_mode_entry>>,
}

#[derive(Default)]
#[repr(C)]
pub struct window_pane_modes {
    pub storage: Option<Box<WindowPaneModesStorage>>,
}

impl window_pane_modes {
    pub fn is_empty(&self) -> bool {
        self.storage.as_ref().is_none_or(|storage| storage.entries.is_empty())
    }

    pub fn active_mode(&self) -> Option<&'static window_mode> {
        let entry = self.storage.as_ref()?.entries.first()?;
        Some(entry.try_borrow_mut().expect("active pane mode already borrowed").mode)
    }

    /// Observe the current entry without retaining the pane-owned allocation.
    pub fn active_weak(&self) -> refbox::Weak<window_mode_entry> {
        self.storage
            .as_ref()
            .and_then(|storage| storage.entries.first())
            .map_or_else(refbox::Weak::new, refbox::RefBox::downgrade)
    }

    /// A borrowed compatibility pointer valid until the mode stack changes.
    pub fn active_ptr(&self) -> *mut window_mode_entry {
        self.storage
            .as_ref()
            .and_then(|storage| storage.entries.first())
            .map_or(std::ptr::null_mut(), |entry| entry.as_ptr().cast_mut())
    }
}

/// Ordered, non-owning pane handles. Retained Rc references keep allocations
/// alive; this collection only records order.
#[derive(Default)]
pub struct window_panes {
    pub storage: Option<Box<Vec<std::rc::Weak<std::cell::UnsafeCell<window_pane>>>>>,
}

impl window_panes {
    /// Retain every pane in display order for operations that may outlive membership.
    pub fn snapshot(&self) -> Vec<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage.as_deref().into_iter().flatten()
            .map(|observer| observer.upgrade().expect("live pane in ordering"))
            .collect()
    }

    pub fn first(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .as_deref()
            .and_then(|panes| panes.first())
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn next(&self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        let Some(storage) = self.storage.as_deref() else {
            return None;
        };
        let Some(position) = storage
            .iter()
            .position(|weak| weak.ptr_eq(pane))
        else {
            return None;
        };
        storage
            .get(position + 1)
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn last(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .as_deref()
            .and_then(|panes| panes.last())
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn previous(&self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.position(pane)
            .and_then(|position| position.checked_sub(1))
            .and_then(|position| self.storage.as_deref()?.get(position))
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn position(&self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> Option<usize> {
        self.storage
            .as_deref()?
            .iter()
            .position(|weak| weak.ptr_eq(pane))
    }

    pub fn push_front(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        self.storage
            .get_or_insert_with(|| Box::new(Vec::new()))
            .insert(0, pane);
    }

    pub fn push_back(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        self.storage
            .get_or_insert_with(|| Box::new(Vec::new()))
            .push(pane);
    }

    pub fn insert_before(
        &mut self,
        before: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        let position = self
            .position(before)
            .expect("insertion point is not in pane collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .insert(position, pane);
    }

    pub fn insert_after(
        &mut self,
        after: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        let position = self
            .position(after)
            .expect("insertion point is not in pane collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .insert(position + 1, pane);
    }

    pub fn remove(&mut self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        let Some(position) = storage
            .iter()
            .position(|weak| weak.ptr_eq(pane))
        else {
            return false;
        };
        storage.remove(position);
        if storage.is_empty() {
            self.storage = None;
        }
        true
    }

    pub fn swap(&mut self, first: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>, second: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        let first_position = self
            .position(first)
            .expect("first pane is not in collection");
        let second_position = self
            .position(second)
            .expect("second pane is not in collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .swap(first_position, second_position);
    }

    pub fn remove_at(
        &mut self,
        pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
        let position = self.position(pane).expect("pane is not in collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .remove(position)
    }

    pub fn insert_at(
        &mut self,
        position: usize,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .insert(position, pane);
    }
}

/// Most-recently-visited pane handles, with the newest pane at the front.
#[derive(Default)]
pub struct window_pane_history {
    pub storage: Option<Box<VecDeque<std::rc::Weak<std::cell::UnsafeCell<window_pane>>>>>,
}

impl window_pane_history {
    pub fn is_empty(&self) -> bool {
        self.storage.as_deref().is_none_or(VecDeque::is_empty)
    }

    pub fn first(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .as_deref()
            .and_then(|panes| panes.front())
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn next(&self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        let Some(storage) = self.storage.as_deref() else {
            return None;
        };
        let Some(position) = storage
            .iter()
            .position(|weak| weak.ptr_eq(pane))
        else {
            return None;
        };
        storage
            .get(position + 1)
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn remove(&mut self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        let old_len = storage.len();
        storage.retain(|weak| !weak.ptr_eq(pane));
        if storage.is_empty() {
            self.storage = None;
        }
        old_len != self.storage.as_ref().map_or(0, |value| value.len())
    }

    pub fn push_front(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        self.storage
            .get_or_insert_with(|| Box::new(VecDeque::new()))
            .push_front(pane);
    }
}

#[repr(C)]
pub struct window_pane_tree {
    /// The global pane index owns both its map and the Rc pane records.
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<u_int, std::rc::Rc<std::cell::UnsafeCell<window_pane>>>>>,
}

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
use super::prompt::{prompt_free_cb, prompt_type, PromptOwner};
use super::screen::screen;
use super::spawn::spawn_editor_state;
use super::status::status_prompt_input_cb;
use super::style::{style, style_line_entry};
use super::window::{window, window_mode_entry};
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct window_pane_offset {
    pub used: size_t,
}
#[repr(C)]
pub struct window_pane_resizes {
    /// The queue owner is optional so an empty queue has no heap allocation.
    /// Keep the reserved ABI slot until the containing pane is fully migrated.
    pub storage: Option<Box<window_pane_resize_storage>>,
    pub reserved: *mut ::core::ffi::c_void,
}

/// Each entry remains separately boxed so pointers used by cancellation keep
/// their address when the deque grows or other entries are removed.
pub type window_pane_resize_storage = VecDeque<Box<window_pane_resize>>;

impl Default for window_pane_resizes {
    fn default() -> Self {
        Self {
            storage: None,
            reserved: ::core::ptr::null_mut(),
        }
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
pub type WindowPanePromptOwner = refbox::RefBox<window_pane_prompt>;
pub type WindowPanePromptWeak = Option<refbox::Weak<window_pane_prompt>>;

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
    pub id: u_int,
    pub active_point: u_int,
    pub window: *mut window,
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
    pub wait_item: *mut cmdq_item,
    pub editor: Option<Box<spawn_editor_state>>,
    pub output_generation: uint64_t,
    pub last_output_time: time_t,
    pub last_prompt_time: time_t,
    pub cmd_start_time: time_t,
    pub cmd_end_time: time_t,
    pub cmd_status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub event: *mut bufferevent,
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
    pub pipe_event: *mut bufferevent,
    pub pipe_offset: window_pane_offset,
    pub screen: *mut screen,
    pub base: screen,
    pub status_screen: screen,
    /// Pane-owned non-intrusive mode stack, drained before pane free.
    pub modes: window_pane_modes,
    pub searchstr: Option<CString>,
    pub searchregex: ::core::ffi::c_int,
    pub prompt: Option<PromptOwner>,
    /// Callback-owned data; this observer cannot keep a closed prompt alive.
    pub prompt_data: WindowPanePromptWeak,
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
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Default)]
#[repr(C)]
pub struct window_pane_tree_entry {
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<u_int, *mut window_pane>>>,
}

/// Ordered mode stack owned by a pane. Each entry remains boxed so mode
/// callbacks and pending events can keep stable pointers while the stack moves.
#[derive(Default)]
pub struct WindowPaneModesStorage {
    pub(crate) entries: Vec<Box<window_mode_entry>>,
}

#[repr(C)]
pub struct window_pane_modes {
    /// Compatibility view of the top mode; the storage owns every entry.
    pub active: *mut window_mode_entry,
    pub storage: Option<Box<WindowPaneModesStorage>>,
}

impl Default for window_pane_modes {
    fn default() -> Self {
        Self {
            active: std::ptr::null_mut(),
            storage: None,
        }
    }
}

/// Ordered, non-owning pane handles. Retained Rc references keep allocations
/// alive; this collection only records order.
#[derive(Default)]
pub struct window_panes {
    pub storage: Option<Box<Vec<std::rc::Weak<std::cell::UnsafeCell<window_pane>>>>>,
}

fn checked_window_pane_ptr(
    weak: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
) -> *mut window_pane {
    let owner = weak
        .upgrade()
        .expect("window pane collection contains an expired owner");
    super::rc::as_ptr(&owner)
}

impl window_panes {
    pub unsafe fn first(&self) -> *mut window_pane {
        self.storage
            .as_deref()
            .and_then(|panes| panes.first())
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn next(&self, pane: *mut window_pane) -> *mut window_pane {
        let Some(storage) = self.storage.as_deref() else {
            return std::ptr::null_mut();
        };
        let Some(position) = storage
            .iter()
            .position(|weak| checked_window_pane_ptr(weak) == pane)
        else {
            return std::ptr::null_mut();
        };
        storage
            .get(position + 1)
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn last(&self) -> *mut window_pane {
        self.storage
            .as_deref()
            .and_then(|panes| panes.last())
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn previous(&self, pane: *mut window_pane) -> *mut window_pane {
        self.position(pane)
            .and_then(|position| position.checked_sub(1))
            .and_then(|position| self.storage.as_deref()?.get(position))
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn position(&self, pane: *mut window_pane) -> Option<usize> {
        self.storage
            .as_deref()?
            .iter()
            .position(|weak| checked_window_pane_ptr(weak) == pane)
    }

    pub fn push_front(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        unsafe {
            self.remove_ptr(checked_window_pane_ptr(&pane));
        }
        self.storage
            .get_or_insert_with(|| Box::new(Vec::new()))
            .insert(0, pane);
    }

    pub fn push_back(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        unsafe {
            self.remove_ptr(checked_window_pane_ptr(&pane));
        }
        self.storage
            .get_or_insert_with(|| Box::new(Vec::new()))
            .push(pane);
    }

    pub unsafe fn insert_before(
        &mut self,
        before: *mut window_pane,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        let pointer = checked_window_pane_ptr(&pane);
        self.remove_ptr(pointer);
        let position = self
            .position(before)
            .expect("insertion point is not in pane collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .insert(position, pane);
    }

    pub unsafe fn insert_after(
        &mut self,
        after: *mut window_pane,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        let pointer = checked_window_pane_ptr(&pane);
        self.remove_ptr(pointer);
        let position = self
            .position(after)
            .expect("insertion point is not in pane collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .insert(position + 1, pane);
    }

    pub unsafe fn remove_ptr(&mut self, pane: *mut window_pane) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        let Some(position) = storage
            .iter()
            .position(|weak| checked_window_pane_ptr(weak) == pane)
        else {
            return false;
        };
        storage.remove(position);
        if storage.is_empty() {
            self.storage = None;
        }
        true
    }

    pub unsafe fn swap_ptrs(&mut self, first: *mut window_pane, second: *mut window_pane) {
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

    pub unsafe fn remove_at(
        &mut self,
        pane: *mut window_pane,
    ) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
        let position = self.position(pane).expect("pane is not in collection");
        self.storage
            .as_mut()
            .expect("pane collection is present")
            .remove(position)
    }

    pub unsafe fn insert_at(
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

    pub unsafe fn first(&self) -> *mut window_pane {
        self.storage
            .as_deref()
            .and_then(|panes| panes.front())
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn next(&self, pane: *mut window_pane) -> *mut window_pane {
        let Some(storage) = self.storage.as_deref() else {
            return std::ptr::null_mut();
        };
        let Some(position) = storage
            .iter()
            .position(|weak| checked_window_pane_ptr(weak) == pane)
        else {
            return std::ptr::null_mut();
        };
        storage
            .get(position + 1)
            .map_or(std::ptr::null_mut(), checked_window_pane_ptr)
    }

    pub unsafe fn remove_ptr(&mut self, pane: *mut window_pane) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        let old_len = storage.len();
        storage.retain(|weak| checked_window_pane_ptr(weak) != pane);
        if storage.is_empty() {
            self.storage = None;
        }
        old_len != self.storage.as_ref().map_or(0, |value| value.len())
    }

    pub fn push_front(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        let pointer = checked_window_pane_ptr(&pane);
        unsafe {
            self.remove_ptr(pointer);
        }
        self.storage
            .get_or_insert_with(|| Box::new(VecDeque::new()))
            .push_front(pane);
    }
}

#[repr(C)]
pub struct window_pane_tree {
    /// The global pane index owns its map; retained Rc references own pane records.
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<u_int, *mut window_pane>>>,
}

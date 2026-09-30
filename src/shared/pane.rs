//! Authoritative pane declarations, shared by the C translation units.

use crate::src::shared::client::ClientWeak;
use crate::src::shared::window::{WindowRef, WindowWeak};
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ffi::CString;
use std::rc::{Rc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

use super::abi::{bitstr_t, pid_t, size_t, time_t, u_int, uint64_t};
use super::client::client;
use super::colour::{client_theme, colour_palette};
use super::command::cmdq_item;
use super::event::{bufferevent, event};
use super::grid::grid_cell;
use super::input::input_ctx;
use super::layout::LayoutCellId;
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

/// Boxes preserve resize addresses while the queue grows or removes entries.
pub type window_pane_resizes = VecDeque<Box<window_pane_resize>>;

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
    pub c: ClientWeak,
    pub inputcb: status_prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub type_0: prompt_type,
}

#[repr(C)]
pub struct window_pane {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub id: u_int,
    pub active_point: u_int,
    /// Nonowning parent; final window teardown provides a scoped fallback.
    pub window: WindowWeak,
    pub options: Option<Box<options>>,
    /// Nonowning cell identities; resolve only under the owning Window tree guard.
    pub layout_cell: Option<LayoutCellId>,
    pub saved_layout_cell: Option<LayoutCellId>,
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
    pub sb_auto_timer: event,
    pub argv: Vec<std::ffi::CString>,
    pub shell: Option<CString>,
    pub cwd: Option<CString>,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub dead_time: SystemTime,
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
    /// Weak traversal handle into the containing index.
    pub owner: refbox::Weak<
        std::collections::BTreeMap<u_int, std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    >,
}
impl Default for window_pane {
    fn default() -> Self {
        Self {
            observer: Default::default(),
            id: Default::default(),
            active_point: Default::default(),
            window: Default::default(),
            options: Default::default(),
            layout_cell: Default::default(),
            saved_layout_cell: Default::default(),
            sx: Default::default(),
            sy: Default::default(),
            xoff: Default::default(),
            yoff: Default::default(),
            flags: Default::default(),
            sync_dirty: Default::default(),
            sync_dirty_size: Default::default(),
            sb_slider_y: Default::default(),
            sb_slider_h: Default::default(),
            sb_auto_visible: Default::default(),
            sb_auto_timer: Default::default(),
            argv: Default::default(),
            shell: Default::default(),
            cwd: Default::default(),
            pid: Default::default(),
            tty: Default::default(),
            status: Default::default(),
            dead_time: UNIX_EPOCH,
            wait_item: Default::default(),
            editor: Default::default(),
            output_generation: Default::default(),
            last_output_time: Default::default(),
            last_prompt_time: Default::default(),
            cmd_start_time: Default::default(),
            cmd_end_time: Default::default(),
            cmd_status: Default::default(),
            fd: Default::default(),
            event: Default::default(),
            offset: Default::default(),
            base_offset: Default::default(),
            resize_queue: Default::default(),
            resize_timer: Default::default(),
            sync_timer: Default::default(),
            ictx: Default::default(),
            cached_gc: Default::default(),
            cached_active_gc: Default::default(),
            cached_dim: Default::default(),
            cached_active_dim: Default::default(),
            palette: Default::default(),
            last_theme: Default::default(),
            border_status_line: Default::default(),
            pipe_fd: Default::default(),
            pipe_pid: Default::default(),
            pipe_event: Default::default(),
            pipe_offset: Default::default(),
            screen_source: Default::default(),
            base: Default::default(),
            status_screen: Default::default(),
            modes: Default::default(),
            searchstr: Default::default(),
            searchregex: Default::default(),
            prompt: Default::default(),
            prompt_data: Default::default(),
            prompt_cx: Default::default(),
            border_gc_set: Default::default(),
            border_gc: Default::default(),
            active_border_gc_set: Default::default(),
            active_border_gc: Default::default(),
            control_bg: Default::default(),
            control_fg: Default::default(),
            scrollbar_style: Default::default(),
            owner: Default::default(),
        }
    }
}

impl window_pane {
    /// Retain the parent for the current operation. Release potentially final
    /// owners through window_remove_ref, as with other window handles.
    pub fn window_handle(&self) -> Option<WindowRef> {
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
                let mode = observer.get_unchecked().mode;
                mode.display_screen.expect("mode display screen getter")(observer.clone())
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

/// Pane-owned mode entries stay allocated until explicit mode cleanup removes them.
pub type window_pane_modes = Vec<refbox::RefBox<window_mode_entry>>;

impl window_pane {
    pub fn active_mode(&self) -> Option<&'static window_mode> {
        let entry = self.modes.first()?;
        Some(
            entry
                .try_borrow_mut()
                .expect("active pane mode already borrowed")
                .mode,
        )
    }

    /// Observe the current entry without retaining the pane-owned allocation.
    pub fn active_mode_entry(&self) -> refbox::Weak<window_mode_entry> {
        self.modes
            .first()
            .map_or_else(refbox::Weak::new, refbox::RefBox::downgrade)
    }
}

/// Ordered, non-owning pane handles. Retained Rc references keep allocations
/// alive; this collection only records order.
#[derive(Default)]
pub struct window_panes {
    pub storage: Vec<std::rc::Weak<std::cell::UnsafeCell<window_pane>>>,
}

impl window_panes {
    /// Retain every pane in display order for operations that may outlive membership.
    pub fn snapshot(&self) -> Vec<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .iter()
            .map(|observer| observer.upgrade().expect("live pane in ordering"))
            .collect()
    }

    pub fn first(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .first()
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn next(
        &self,
        pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        let storage = &self.storage;
        let Some(position) = storage.iter().position(|weak| weak.ptr_eq(pane)) else {
            return None;
        };
        storage
            .get(position + 1)
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn last(&self) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.storage
            .last()
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn previous(
        &self,
        pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
        self.position(pane)
            .and_then(|position| position.checked_sub(1))
            .and_then(|position| self.storage.get(position))
            .map(|weak| weak.upgrade().expect("live pane in ordering"))
    }

    pub fn position(
        &self,
        pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) -> Option<usize> {
        self.storage.iter().position(|weak| weak.ptr_eq(pane))
    }

    pub fn push_front(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        self.storage.insert(0, pane);
    }

    pub fn push_back(&mut self, pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>) {
        assert!(pane.strong_count() != 0, "live pane for insertion");
        self.remove(&pane);
        self.storage.push(pane);
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
        self.storage.insert(position, pane);
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
        self.storage.insert(position + 1, pane);
    }

    pub fn remove(&mut self, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> bool {
        let storage = &mut self.storage;
        let Some(position) = storage.iter().position(|weak| weak.ptr_eq(pane)) else {
            return false;
        };
        storage.remove(position);
        true
    }

    pub fn swap(
        &mut self,
        first: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
        second: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        let first_position = self
            .position(first)
            .expect("first pane is not in collection");
        let second_position = self
            .position(second)
            .expect("second pane is not in collection");
        self.storage.swap(first_position, second_position);
    }

    pub fn remove_at(
        &mut self,
        pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
        let position = self.position(pane).expect("pane is not in collection");
        self.storage.remove(position)
    }

    pub fn insert_at(
        &mut self,
        position: usize,
        pane: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    ) {
        self.storage.insert(position, pane);
    }
}

/// Most-recently-visited pane handles, with the newest pane at the front.
pub type window_pane_history = VecDeque<std::rc::Weak<std::cell::UnsafeCell<window_pane>>>;

#[repr(C)]
pub struct window_pane_tree {
    /// The global pane index owns both its map and the Rc pane records.
    pub storage: Option<
        refbox::RefBox<
            std::collections::BTreeMap<u_int, std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
        >,
    >,
}

impl window_pane {
    /// Append a separately allocated resize and return its stable address.
    pub fn push_resize(&mut self, resize: window_pane_resize) -> *mut window_pane_resize {
        let resize = Box::new(resize);
        let pointer = (&*resize) as *const window_pane_resize as *mut window_pane_resize;
        self.resize_queue.push_back(resize);
        pointer
    }

    /// Retain only `except`; removed entry pointers are invalid after return.
    pub fn clear_resizes_except(&mut self, except: *mut window_pane_resize) {
        if except.is_null() {
            self.resize_queue = Default::default();
            return;
        }
        self.resize_queue
            .retain(|resize| std::ptr::eq(&**resize, except));
    }
}

pub fn pane_history_first(history: &window_pane_history) -> Option<Rc<UnsafeCell<window_pane>>> {
    history
        .front()
        .map(|weak| weak.upgrade().expect("live pane in ordering"))
}

pub fn pane_history_next(
    history: &window_pane_history,
    pane: &Weak<UnsafeCell<window_pane>>,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    let position = history.iter().position(|weak| weak.ptr_eq(pane))?;
    history
        .get(position + 1)
        .map(|weak| weak.upgrade().expect("live pane in ordering"))
}

pub fn pane_history_remove(
    history: &mut window_pane_history,
    pane: &Weak<UnsafeCell<window_pane>>,
) -> bool {
    let old_len = history.len();
    history.retain(|weak| !weak.ptr_eq(pane));
    old_len != history.len()
}

pub fn pane_history_push(history: &mut window_pane_history, pane: Weak<UnsafeCell<window_pane>>) {
    assert!(pane.strong_count() != 0, "live pane for insertion");
    pane_history_remove(history, &pane);
    history.push_front(pane);
}

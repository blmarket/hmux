//! Pane storage belongs to the pane implementation.
use super::*;
use crate::src::shared::window::WindowWeak;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub sb_auto_timer: Timer,
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
    pub resize_timer: Timer,
    pub sync_timer: Timer,
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
    /// owners through Window::release, as with other window handles.
    pub(super) fn window_handle(&self) -> Option<WindowRef> {
        self.window.upgrade()
    }

    /// Resolve the displayed screen while the pane and selected mode are live.
    pub(super) unsafe fn screen_ptr(&self) -> *mut screen {
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

    pub(super) fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        Self::empty().into_shared()
    }

    pub(super) fn into_shared(mut self) -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new_cyclic(|observer| {
            self.observer = observer.clone();
            std::cell::UnsafeCell::new(self)
        })
    }

    pub(super) fn empty() -> Self {
        Self::default()
    }
}

impl window_pane {
    pub(super) fn active_mode(&self) -> Option<&'static window_mode> {
        let entry = self.modes.first()?;
        Some(
            entry
                .try_borrow_mut()
                .expect("active pane mode already borrowed")
                .mode,
        )
    }

    /// Observe the current entry without retaining the pane-owned allocation.
    pub(super) fn active_mode_entry(&self) -> refbox::Weak<window_mode_entry> {
        self.modes
            .first()
            .map_or_else(refbox::Weak::new, refbox::RefBox::downgrade)
    }
}

impl window_pane {
    /// Append a separately allocated resize and return its stable address.
    pub(super) fn push_resize(&mut self, resize: window_pane_resize) -> *mut window_pane_resize {
        let resize = Box::new(resize);
        let pointer = (&*resize) as *const window_pane_resize as *mut window_pane_resize;
        self.resize_queue.push_back(resize);
        pointer
    }

    /// Retain only `except`; removed entry pointers are invalid after return.
    pub(super) fn clear_resizes_except(&mut self, except: *mut window_pane_resize) {
        if except.is_null() {
            self.resize_queue = Default::default();
            return;
        }
        self.resize_queue
            .retain(|resize| std::ptr::eq(&**resize, except));
    }
}

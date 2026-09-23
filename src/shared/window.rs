//! Authoritative window declarations, shared by the C translation units.

use super::abi::{timeval, u_int, uint64_t};
use super::arguments::args;
use super::client::client;
use super::command::{cmd_find_state, cmdq_item};
use super::event::event;
use super::format::format_tree;
use super::grid::grid_cell;
use super::key::{key_code, key_table};
use super::layout::layout_cell;
use super::menu::{menu, menu_data};
use super::mouse::mouse_event;
use super::options::options;
use super::pane::{window_pane, window_panes, PANE_MINIMUM};
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
    pub storage:
        Option<Box<std::collections::BTreeMap<::core::ffi::c_int, refbox::RefBox<winlink>>>>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink {
    pub idx: ::core::ffi::c_int,
    pub session: *mut session,
    pub window: *mut window,
    pub flags: ::core::ffi::c_int,
    pub entry: winlink_entry,
    pub wentry: winlink_wentry,
    pub sentry: winlink_sentry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink_sentry {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink_wentry {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink_entry {
    pub owner: *mut std::collections::BTreeMap<::core::ffi::c_int, refbox::RefBox<winlink>>,
}

#[repr(C)]
/// Box-owned window record; references remain managed by window_add_ref/remove_ref.
pub struct window {
    pub id: u_int,
    pub latest: *mut ::core::ffi::c_void,
    pub name: *mut ::core::ffi::c_char,
    pub name_event: event,
    pub name_time: timeval,
    pub alerts_timer: event,
    pub offset_timer: event,
    pub activity_time: timeval,
    pub creation_time: timeval,
    pub active: *mut window_pane,
    pub modal: *mut window_pane,
    pub modal_last: *mut window_pane,
    pub was_zoomed: *mut window_pane,
    pub last_panes: window_panes,
    pub z_index: window_panes,
    pub panes: window_panes,
    pub lastlayout: ::core::ffi::c_int,
    pub layout_root: *mut layout_cell,
    pub saved_layout_root: *mut layout_cell,
    pub old_layout: *mut ::core::ffi::c_char,
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
    pub menu: *mut menu_data,
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
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: window_winlinks,
    pub entry: window_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_entry {
    /// Compatibility view of the owning boxed index; null after removal.
    pub owner: *mut std::collections::BTreeMap<u_int, *mut window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_winlinks {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_alerts_entry {
    pub tqe_next: *mut window,
    pub tqe_prev: *mut *mut window,
}

#[repr(C)]
pub struct window_mode_entry {
    pub wp: *mut window_pane,
    pub swp: *mut window_pane,
    pub mode: *const window_mode,
    pub data: *mut ::core::ffi::c_void,
    pub screen: *mut screen,
    pub prefix: u_int,
    pub kill: ::core::ffi::c_int,
    pub entry: window_mode_entry_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode_entry_entry {
    pub tqe_next: *mut window_mode_entry,
    pub tqe_prev: *mut *mut window_mode_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode {
    pub name: *const ::core::ffi::c_char,
    pub default_format: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub init: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut cmdq_item,
            *mut cmd_find_state,
            *mut args,
        ) -> *mut screen,
    >,
    pub free: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub resize: Option<unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> ()>,
    pub update: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub style_changed: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub key: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            key_code,
            *mut mouse_event,
        ) -> (),
    >,
    pub key_table:
        Option<unsafe extern "C" fn(*mut window_mode_entry) -> *const ::core::ffi::c_char>,
    pub command: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            *mut args,
            *mut mouse_event,
        ) -> (),
    >,
    pub formats: Option<unsafe extern "C" fn(*mut window_mode_entry, *mut format_tree) -> ()>,
    pub get_screen: Option<unsafe extern "C" fn(*mut window_mode_entry) -> *mut screen>,
}

#[repr(C)]
pub struct winlink_stack {
    /// Weak visit history; the session owns the deque and the ordered index
    /// owns each link.
    pub storage: Option<Box<std::collections::VecDeque<refbox::Weak<winlink>>>>,
    /// Reserved ABI slot for the translated layout.
    pub reserved: *mut ::core::ffi::c_void,
}

#[repr(C)]
pub struct windows {
    /// The head owns the index allocation; window records remain externally owned.
    pub storage: Option<Box<std::collections::BTreeMap<u_int, *mut window>>>,
}

//! The [`window_mode`] table behind each [`WindowMode`].
//!
//! Each pane mode used to be a `static` table of function pointers that
//! `window_mode_entry` pointed at, and a mode was identified by that pointer.
//! The tables are still whole — one `const` per mode — but they are now
//! reached through the variant that names them, so [`WindowMode::table`] is
//! the single place mapping a mode to what it implements.
//!
//! The methods below are conveniences over that table: they are what call
//! sites use, so a slot the mode left empty reads as nothing to do rather than
//! as a null check at every caller.

use super::buffer::{
    WINDOW_BUFFER_DEFAULT_FORMAT, window_buffer_free, window_buffer_init, window_buffer_key,
    window_buffer_modedata, window_buffer_resize, window_buffer_update,
};
use super::client::{
    WINDOW_CLIENT_DEFAULT_FORMAT, window_client_free, window_client_init, window_client_key,
    window_client_modedata, window_client_resize, window_client_update,
};
use super::clock::{
    window_clock_free, window_clock_init, window_clock_key, window_clock_mode_data,
    window_clock_resize,
};
use super::copy::{
    window_copy_command, window_copy_formats, window_copy_free, window_copy_get_screen,
    window_copy_init, window_copy_key_table, window_copy_mode_data, window_copy_resize,
    window_copy_style_changed, window_copy_view_init,
};
use super::customize::{
    WINDOW_CUSTOMIZE_DEFAULT_FORMAT, window_customize_free, window_customize_init,
    window_customize_key, window_customize_modedata, window_customize_resize,
};
use super::tree::{
    WINDOW_TREE_DEFAULT_FORMAT, window_tree_free, window_tree_init, window_tree_key,
    window_tree_modedata, window_tree_resize, window_tree_update,
};
use crate::types::*;

impl window_mode {
    /// A table implementing nothing, over which each mode's own table fills in
    /// the slots it has.
    pub const NONE: window_mode = window_mode {
        name: c"",
        default_format: None,
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
    };
}

impl WindowModeData {
    /// The buffer mode this names, or null once it has gone.
    pub fn buffer(&self) -> *mut window_buffer_modedata {
        match self {
            WindowModeData::Buffer(held) => held
                .upgrade()
                .map_or(::core::ptr::null_mut(), |held| held.as_ptr()),
            WindowModeData::None => ::core::ptr::null_mut(),
            _ => panic!("not buffer-mode state"),
        }
    }

    /// The client mode this names, or null once it has gone.
    pub fn client(&self) -> *mut window_client_modedata {
        match self {
            WindowModeData::Client(held) => held
                .upgrade()
                .map_or(::core::ptr::null_mut(), |held| held.as_ptr()),
            WindowModeData::None => ::core::ptr::null_mut(),
            _ => panic!("not client-mode state"),
        }
    }

    /// The tree mode this names, or null once it has gone.
    pub fn tree(&self) -> *mut window_tree_modedata {
        match self {
            WindowModeData::Tree(held) => held
                .upgrade()
                .map_or(::core::ptr::null_mut(), |held| held.as_ptr()),
            WindowModeData::None => ::core::ptr::null_mut(),
            _ => panic!("not tree-mode state"),
        }
    }

    /// The options mode this names, or null once it has gone.
    pub fn customize(&self) -> *mut window_customize_modedata {
        match self {
            WindowModeData::Customize(held) => held
                .upgrade()
                .map_or(::core::ptr::null_mut(), |held| held.as_ptr()),
            WindowModeData::None => ::core::ptr::null_mut(),
            _ => panic!("not options-mode state"),
        }
    }
}

impl WindowModeState {
    pub(crate) fn mode(&self) -> Option<WindowMode> {
        match self {
            WindowModeState::None => None,
            WindowModeState::Clock(_) => Some(WindowMode::Clock),
            WindowModeState::Copy(_) => Some(WindowMode::Copy),
            WindowModeState::View(_) => Some(WindowMode::View),
            WindowModeState::Buffer(_) => Some(WindowMode::Buffer),
            WindowModeState::Client(_) => Some(WindowMode::Client),
            WindowModeState::Tree(_) => Some(WindowMode::Tree),
            WindowModeState::Customize(_) => Some(WindowMode::Customize),
        }
    }

    /// The state [`window_clock_init`] built.
    pub(crate) fn clock(&self) -> *mut window_clock_mode_data {
        match self {
            WindowModeState::Clock(data) => {
                data.as_ref() as *const window_clock_mode_data as *mut window_clock_mode_data
            }
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not clock-mode state"),
        }
    }

    /// The state [`window_copy_init`] built, shared by `copy-mode` and
    /// `view-mode`.
    pub(crate) fn copy(&self) -> *mut window_copy_mode_data {
        match self {
            WindowModeState::Copy(data) | WindowModeState::View(data) => {
                data.as_ref() as *const window_copy_mode_data as *mut window_copy_mode_data
            }
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not copy-mode state"),
        }
    }

    /// The state [`window_buffer_init`] built.
    pub(crate) fn buffer(&self) -> *mut window_buffer_modedata {
        match self {
            WindowModeState::Buffer(data) => data.as_ptr(),
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not buffer-mode state"),
        }
    }

    /// The state [`window_client_init`] built.
    pub(crate) fn client(&self) -> *mut window_client_modedata {
        match self {
            WindowModeState::Client(data) => data.as_ptr(),
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not client-mode state"),
        }
    }

    /// The state [`window_tree_init`] built.
    pub(crate) fn tree(&self) -> *mut window_tree_modedata {
        match self {
            WindowModeState::Tree(data) => data.as_ptr(),
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not tree-mode state"),
        }
    }

    /// The state [`window_customize_init`] built.
    pub(crate) fn customize(&self) -> *mut window_customize_modedata {
        match self {
            WindowModeState::Customize(data) => data.as_ptr(),
            WindowModeState::None => ::core::ptr::null_mut(),
            _ => panic!("not options-mode state"),
        }
    }
}

impl window_mode_entry {
    pub(crate) fn mode(&self) -> WindowMode {
        self.state.mode().expect("window mode entry has no state")
    }
}

impl WindowMode {
    const CLOCK: window_mode = window_mode {
        name: c"clock-mode",
        init: Some(window_clock_init),
        free: Some(window_clock_free),
        resize: Some(window_clock_resize),
        key: Some(window_clock_key),
        ..window_mode::NONE
    };

    const COPY: window_mode = window_mode {
        name: c"copy-mode",
        init: Some(window_copy_init),
        free: Some(window_copy_free),
        resize: Some(window_copy_resize),
        style_changed: Some(window_copy_style_changed),
        key_table: Some(window_copy_key_table),
        command: Some(window_copy_command),
        formats: Some(window_copy_formats),
        get_screen: Some(window_copy_get_screen),
        ..window_mode::NONE
    };

    const VIEW: window_mode = window_mode {
        name: c"view-mode",
        init: Some(window_copy_view_init),
        ..WindowMode::COPY
    };

    const BUFFER: window_mode = window_mode {
        name: c"buffer-mode",
        default_format: Some(WINDOW_BUFFER_DEFAULT_FORMAT),
        init: Some(window_buffer_init),
        free: Some(window_buffer_free),
        resize: Some(window_buffer_resize),
        update: Some(window_buffer_update),
        key: Some(window_buffer_key),
        ..window_mode::NONE
    };

    const CLIENT: window_mode = window_mode {
        name: c"client-mode",
        default_format: Some(WINDOW_CLIENT_DEFAULT_FORMAT),
        init: Some(window_client_init),
        free: Some(window_client_free),
        resize: Some(window_client_resize),
        update: Some(window_client_update),
        key: Some(window_client_key),
        ..window_mode::NONE
    };

    const TREE: window_mode = window_mode {
        name: c"tree-mode",
        default_format: Some(WINDOW_TREE_DEFAULT_FORMAT),
        init: Some(window_tree_init),
        free: Some(window_tree_free),
        resize: Some(window_tree_resize),
        update: Some(window_tree_update),
        key: Some(window_tree_key),
        ..window_mode::NONE
    };

    const CUSTOMIZE: window_mode = window_mode {
        name: c"options-mode",
        default_format: Some(WINDOW_CUSTOMIZE_DEFAULT_FORMAT),
        init: Some(window_customize_init),
        free: Some(window_customize_free),
        resize: Some(window_customize_resize),
        key: Some(window_customize_key),
        ..window_mode::NONE
    };

    /// What this mode implements.
    pub const fn table(self) -> &'static window_mode {
        match self {
            WindowMode::Clock => &WindowMode::CLOCK,
            WindowMode::Copy => &WindowMode::COPY,
            WindowMode::View => &WindowMode::VIEW,
            WindowMode::Buffer => &WindowMode::BUFFER,
            WindowMode::Client => &WindowMode::CLIENT,
            WindowMode::Tree => &WindowMode::TREE,
            WindowMode::Customize => &WindowMode::CUSTOMIZE,
        }
    }

    /// The mode's name, as `#{pane_mode}` and `choose-tree` report it.
    pub fn name(self) -> &'static ::core::ffi::CStr {
        self.table().name
    }

    /// The mode's built-in format, or nothing for the modes that have none.
    pub fn default_format(self) -> Option<&'static ::core::ffi::CStr> {
        self.table().default_format
    }

    /// Builds the mode's private state and returns the screen it draws on.
    pub unsafe fn init(
        self,
        wme: &mut window_mode_entry,
        fs: *mut cmd_find_state,
        args: Option<&args>,
    ) -> *mut screen {
        unsafe { self.table().init.expect("every mode opens")(wme, fs, args) }
    }

    /// Releases the private state built by [`WindowMode::init`].
    pub unsafe fn free(self, wme: *mut window_mode_entry) {
        unsafe { self.table().free.expect("every mode closes")(&mut *wme) }
    }

    pub unsafe fn resize(self, wme: *mut window_mode_entry, sx: u_int, sy: u_int) {
        unsafe {
            if let Some(f) = self.table().resize {
                f(&mut *wme, sx, sy);
            }
        }
    }

    /// Refreshes a list mode after the things it lists have changed. The modes
    /// that show a fixed pane do nothing.
    pub unsafe fn update(self, wme: *mut window_mode_entry) {
        unsafe {
            if let Some(f) = self.table().update {
                f(&mut *wme);
            }
        }
    }

    pub unsafe fn style_changed(self, wme: *mut window_mode_entry) {
        unsafe {
            if let Some(f) = self.table().style_changed {
                f(&mut *wme);
            }
        }
    }

    /// Whether the mode takes keys itself rather than through a key table.
    pub fn has_key(self) -> bool {
        self.table().key.is_some()
    }

    pub unsafe fn key(
        self,
        wme: *mut window_mode_entry,
        c: *mut client,
        s: *mut session,
        wl: *mut winlink,
        key: key_code,
        m: *mut mouse_event,
    ) {
        unsafe {
            if let Some(f) = self.table().key {
                f(&mut *wme, c, s, wl, key, m);
            }
        }
    }

    /// The key table the mode binds its keys in, or `None` for the modes that
    /// take keys through [`WindowMode::key`] instead.
    pub unsafe fn key_table(
        self,
        wme: *mut window_mode_entry,
    ) -> Option<&'static ::core::ffi::CStr> {
        unsafe { self.table().key_table.map(|f| f(&mut *wme)) }
    }

    /// Whether `send-keys -X` has anything to dispatch to in this mode.
    pub fn has_command(self) -> bool {
        self.table().command.is_some()
    }

    pub unsafe fn command(
        self,
        wme: &mut window_mode_entry,
        c: *mut client,
        s: *mut session,
        wl: *mut winlink,
        args: &args,
        m: Option<&mut mouse_event>,
    ) {
        unsafe {
            if let Some(f) = self.table().command {
                f(wme, c, s, wl, args, m);
            }
        }
    }

    pub unsafe fn formats(self, wme: *mut window_mode_entry, ft: &mut format_tree) {
        unsafe {
            if let Some(f) = self.table().formats {
                f(&mut *wme, ft);
            }
        }
    }

    /// The screen `capture-pane -M` should read, for the modes that keep one
    /// distinct from the pane's own screen.
    pub unsafe fn get_screen(self, wme: *mut window_mode_entry) -> Option<*mut screen> {
        unsafe { self.table().get_screen.map(|f| f(&mut *wme)) }
    }
}

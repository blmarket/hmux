//! The [`window_mode`] table behind each [`WindowMode`].
//!
//! Each pane mode used to be a `static` table of function pointers that
//! `window_mode_entry` pointed at, and a mode was identified by that pointer.
//! Each table requires its lifecycle callbacks and defaults only its optional
//! behavior. The variant names the table, so [`WindowMode::table`] is
//! the single place mapping a mode to what it implements.
//!
//! The methods below are conveniences over that table: they are what call
//! sites use, so a slot the mode left empty reads as nothing to do rather than
//! as a null check at every caller.

use super::buffer::{
    WINDOW_BUFFER_DEFAULT_FORMAT, window_buffer_free, window_buffer_init, window_buffer_resize,
};
use super::client::{
    WINDOW_CLIENT_DEFAULT_FORMAT, window_client_free, window_client_init, window_client_resize,
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
    window_customize_resize,
};
use super::tree::{
    WINDOW_TREE_DEFAULT_FORMAT, window_tree_free, window_tree_init, window_tree_resize,
};
use crate::args::RustArguments;
use crate::types::*;

impl WindowModeData {
    /// The buffer mode this names, or nothing once it has gone.
    pub fn buffer(&self) -> Option<WindowBufferModeDataRef> {
        match self {
            WindowModeData::Buffer(held) => held.upgrade(),
            WindowModeData::None => None,
            _ => panic!("not buffer-mode state"),
        }
    }

    /// The client mode this names, or nothing once it has gone.
    pub fn client(&self) -> Option<WindowClientModeDataRef> {
        match self {
            WindowModeData::Client(held) => held.upgrade(),
            WindowModeData::None => None,
            _ => panic!("not client-mode state"),
        }
    }

    /// The tree mode this names, or nothing once it has gone.
    pub fn tree(&self) -> Option<WindowTreeModeDataRef> {
        match self {
            WindowModeData::Tree(held) => held.upgrade(),
            WindowModeData::None => None,
            _ => panic!("not tree-mode state"),
        }
    }

    /// The options mode this names, or nothing once it has gone.
    pub fn customize(&self) -> Option<WindowCustomizeModeDataRef> {
        match self {
            WindowModeData::Customize(held) => held.upgrade(),
            WindowModeData::None => None,
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

    /// The state [`window_clock_init`] built, or nothing once the mode has
    /// been left.
    pub(crate) fn clock(&mut self) -> Option<&mut window_clock_mode_data> {
        match self {
            WindowModeState::Clock(data) => Some(data),
            WindowModeState::None => None,
            _ => panic!("not clock-mode state"),
        }
    }

    /// Borrows the state shared by `copy-mode` and `view-mode`.
    pub(crate) fn copy_mode_data_ref(&self) -> Option<&window_copy_mode_data> {
        match self {
            WindowModeState::Copy(data) | WindowModeState::View(data) => Some(data),
            WindowModeState::None => None,
            _ => panic!("not copy-mode state"),
        }
    }

    /// Mutably borrows the state shared by `copy-mode` and `view-mode`.
    pub(crate) fn copy_mode_data_mut(&mut self) -> Option<&mut window_copy_mode_data> {
        match self {
            WindowModeState::Copy(data) | WindowModeState::View(data) => Some(data),
            WindowModeState::None => None,
            _ => panic!("not copy-mode state"),
        }
    }

    /// The state [`window_buffer_init`] built.
    pub(crate) fn buffer(&self) -> Option<WindowBufferModeDataRef> {
        match self {
            WindowModeState::Buffer(data) => Some(data.clone()),
            WindowModeState::None => None,
            _ => panic!("not buffer-mode state"),
        }
    }

    /// The state [`window_client_init`] built.
    pub(crate) fn client(&self) -> Option<WindowClientModeDataRef> {
        match self {
            WindowModeState::Client(data) => Some(data.clone()),
            WindowModeState::None => None,
            _ => panic!("not client-mode state"),
        }
    }

    /// The state [`window_tree_init`] built.
    pub(crate) fn tree(&self) -> Option<WindowTreeModeDataRef> {
        match self {
            WindowModeState::Tree(data) => Some(data.clone()),
            WindowModeState::None => None,
            _ => panic!("not tree-mode state"),
        }
    }

    /// The state [`window_customize_init`] built.
    pub(crate) fn customize(&self) -> Option<WindowCustomizeModeDataRef> {
        match self {
            WindowModeState::Customize(data) => Some(data.clone()),
            WindowModeState::None => None,
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
    /// What this mode implements.
    pub fn table(self) -> &'static window_mode {
        static TABLES: [window_mode; 7] = {
            let copy = window_mode {
                name: c"copy-mode",
                init: window_copy_init,
                free: window_copy_free,
                resize: window_copy_resize,
                optional: WindowModeOptional {
                    style_changed: Some(window_copy_style_changed),
                    key_table: Some(window_copy_key_table),
                    command: Some(window_copy_command),
                    formats: Some(window_copy_formats),
                    get_screen: Some(window_copy_get_screen),
                    ..WindowModeOptional::EMPTY
                },
            };
            [
                window_mode {
                    name: c"clock-mode",
                    init: window_clock_init,
                    free: window_clock_free,
                    resize: window_clock_resize,
                    optional: WindowModeOptional::EMPTY,
                },
                copy,
                window_mode {
                    name: c"view-mode",
                    init: window_copy_view_init,
                    ..copy
                },
                window_mode {
                    name: c"buffer-mode",
                    init: window_buffer_init,
                    free: window_buffer_free,
                    resize: window_buffer_resize,
                    optional: WindowModeOptional {
                        default_format: Some(WINDOW_BUFFER_DEFAULT_FORMAT),
                        ..WindowModeOptional::EMPTY
                    },
                },
                window_mode {
                    name: c"client-mode",
                    init: window_client_init,
                    free: window_client_free,
                    resize: window_client_resize,
                    optional: WindowModeOptional {
                        default_format: Some(WINDOW_CLIENT_DEFAULT_FORMAT),
                        ..WindowModeOptional::EMPTY
                    },
                },
                window_mode {
                    name: c"tree-mode",
                    init: window_tree_init,
                    free: window_tree_free,
                    resize: window_tree_resize,
                    optional: WindowModeOptional {
                        default_format: Some(WINDOW_TREE_DEFAULT_FORMAT),
                        ..WindowModeOptional::EMPTY
                    },
                },
                window_mode {
                    name: c"options-mode",
                    init: window_customize_init,
                    free: window_customize_free,
                    resize: window_customize_resize,
                    optional: WindowModeOptional {
                        default_format: Some(WINDOW_CUSTOMIZE_DEFAULT_FORMAT),
                        ..WindowModeOptional::EMPTY
                    },
                },
            ]
        };
        match self {
            WindowMode::Clock => &TABLES[0],
            WindowMode::Copy => &TABLES[1],
            WindowMode::View => &TABLES[2],
            WindowMode::Buffer => &TABLES[3],
            WindowMode::Client => &TABLES[4],
            WindowMode::Tree => &TABLES[5],
            WindowMode::Customize => &TABLES[6],
        }
    }

    /// The mode's name, as `#{pane_mode}` and `choose-tree` report it.
    pub fn name(self) -> &'static core::ffi::CStr {
        self.table().name
    }

    /// The mode's built-in format, or nothing for the modes that have none.
    pub fn default_format(self) -> Option<&'static core::ffi::CStr> {
        self.table().optional.default_format
    }

    /// Builds the mode's private state before making its owned screen available.
    pub(crate) unsafe fn init(
        self,
        wme: &mut window_mode_entry,
        pane: crate::window::RustWindowPaneWeak,
        fs: Option<&cmd_find_state>,
        args: Option<&RustArguments>,
    ) {
        wme.screen = None;
        unsafe { (self.table().init)(wme, pane, fs, args) };
        wme.screen = Some(match &wme.state {
            WindowModeState::Clock(_) => ModeScreen::Clock,
            WindowModeState::Copy(data) | WindowModeState::View(data) => ModeScreen::Shared(data.screen.clone()),
            WindowModeState::Buffer(data) => ModeScreen::Shared(data.borrow().tree_ref().screen_handle().clone()),
            WindowModeState::Client(data) => ModeScreen::Shared(data.borrow().tree_ref().screen_handle().clone()),
            WindowModeState::Tree(data) => ModeScreen::Shared(data.borrow().tree_ref().screen_handle().clone()),
            WindowModeState::Customize(data) => ModeScreen::Shared(data.borrow().tree_ref().screen_handle().clone()),
            WindowModeState::None => return,
        });
    }

    /// Releases the private state built by [`WindowMode::init`].
    pub unsafe fn free(self, wme: &mut window_mode_entry) {
        wme.screen = None;
        unsafe { (self.table().free)(wme) }
    }

    pub unsafe fn resize(self, wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
        unsafe { (self.table().resize)(wme, sx, sy) }
    }

    pub unsafe fn style_changed(self, wme: &mut window_mode_entry) {
        unsafe {
            if let Some(f) = self.table().optional.style_changed {
                f(wme);
            }
        }
    }

    /// Whether the mode takes keys itself rather than through a key table.
    pub fn has_key(self) -> bool {
        !matches!(self, WindowMode::Copy | WindowMode::View)
    }

    /// The key table the mode binds its keys in, or `None` for the modes that
    /// take keys through [`ModeKeyTarget::dispatch`] instead.
    pub unsafe fn key_table(self, wme: &window_mode_entry) -> Option<&'static core::ffi::CStr> {
        unsafe { self.table().optional.key_table.map(|f| f(wme)) }
    }

    /// Whether `send-keys -X` has anything to dispatch to in this mode.
    pub fn has_command(self) -> bool {
        self.table().optional.command.is_some()
    }

    pub unsafe fn command(
        self,
        wme: &mut window_mode_entry,
        c: Option<&mut client>,
        s: Option<&session>,
        wl: Option<&winlink>,
        args: &RustArguments,
        m: Option<&mut mouse_event>,
    ) {
        unsafe {
            if let Some(f) = self.table().optional.command {
                f(wme, c, s, wl, args, m);
            }
        }
    }

    pub unsafe fn formats(self, wme: &window_mode_entry, ft: &mut format_tree) {
        unsafe {
            if let Some(f) = self.table().optional.formats {
                f(wme, ft);
            }
        }
    }

    /// The screen `capture-pane -M` should read, for the modes that keep one
    /// distinct from the pane's own screen.
    pub fn get_screen(self, wme: &window_mode_entry) -> Option<&RustScreen> {
        self.table().optional.get_screen.and_then(|f| f(wme))
    }
}
use crate::screen::RustScreen;

pub(crate) enum ModeKeyTarget {
    Clock(crate::window::RustWindowPaneWeak, TimerHandle),
    Buffer(WindowBufferModeDataRef),
    Client(WindowClientModeDataRef),
    Tree(WindowTreeModeDataRef),
    Customize(WindowCustomizeModeDataRef),
}

impl window_mode_entry {
    pub(crate) fn key_target(&self) -> Option<ModeKeyTarget> {
        match &self.state {
            WindowModeState::Clock(data) => self
                .wp
                .as_ref()
                .filter(|pane| pane.is_alive())
                .map(|pane| ModeKeyTarget::Clock(pane.clone(), data.timer)),
            WindowModeState::Buffer(data) => Some(ModeKeyTarget::Buffer(data.clone())),
            WindowModeState::Client(data) => Some(ModeKeyTarget::Client(data.clone())),
            WindowModeState::Tree(data) => Some(ModeKeyTarget::Tree(data.clone())),
            WindowModeState::Customize(data) => Some(ModeKeyTarget::Customize(data.clone())),
            _ => None,
        }
    }
}

pub(crate) enum ModeUpdateTarget {
    Buffer(WindowBufferModeDataRef),
    Client(WindowClientModeDataRef),
    Tree(WindowTreeModeDataRef),
}

impl window_mode_entry {
    /// Retains the state needed to refresh a list mode after releasing the pane borrow.
    pub(crate) fn update_target(&self) -> Option<ModeUpdateTarget> {
        match &self.state {
            WindowModeState::Buffer(data) => Some(ModeUpdateTarget::Buffer(data.clone())),
            WindowModeState::Client(data) => Some(ModeUpdateTarget::Client(data.clone())),
            WindowModeState::Tree(data) => Some(ModeUpdateTarget::Tree(data.clone())),
            _ => None,
        }
    }
}

impl ModeUpdateTarget {
    pub(crate) unsafe fn dispatch(self) {
        unsafe {
            let pane = match &self {
                Self::Buffer(data) => data.borrow().pane(),
                Self::Client(data) => data.borrow().pane(),
                Self::Tree(data) => data.borrow().pane(),
            };
            let current = pane
                .as_ref()
                .and_then(|pane| pane.get())
                .and_then(|pane| pane.active_mode())
                .is_some_and(|mode| match (&self, &mode.state) {
                    (Self::Buffer(left), WindowModeState::Buffer(right)) => left.ptr_eq(right),
                    (Self::Client(left), WindowModeState::Client(right)) => left.ptr_eq(right),
                    (Self::Tree(left), WindowModeState::Tree(right)) => left.ptr_eq(right),
                    _ => false,
                });
            if !current {
                return;
            }
            match self {
                Self::Buffer(data) => data.update(),
                Self::Client(data) => data.update(),
                Self::Tree(data) => data.update(),
            }
        }
    }
}

impl ModeKeyTarget {
    pub(crate) unsafe fn dispatch(self, c: &mut client, key: key_code, m: Option<&mouse_event>) {
        unsafe {
            let pane = match &self {
                Self::Clock(pane, _) => Some(pane.clone()),
                Self::Buffer(data) => data.borrow().pane(),
                Self::Client(data) => data.borrow().pane(),
                Self::Tree(data) => data.borrow().pane(),
                Self::Customize(data) => data.borrow().pane(),
            };
            let current = pane
                .as_ref()
                .and_then(|pane| pane.get())
                .and_then(|pane| (pane).active_mode())
                .is_some_and(|mode| match (&self, &mode.state) {
                    (Self::Clock(_, timer), WindowModeState::Clock(data)) => *timer == data.timer,
                    (Self::Buffer(left), WindowModeState::Buffer(right)) => left.ptr_eq(right),
                    (Self::Client(left), WindowModeState::Client(right)) => left.ptr_eq(right),
                    (Self::Tree(left), WindowModeState::Tree(right)) => left.ptr_eq(right),
                    (Self::Customize(left), WindowModeState::Customize(right)) => {
                        left.ptr_eq(right)
                    }
                    _ => false,
                });
            if !current {
                return;
            }
            match self {
                Self::Clock(pane, _) => window_clock_key(pane),
                Self::Buffer(data) => data.key(c, key, m),
                Self::Client(data) => data.key(c, key, m),
                Self::Tree(data) => data.key(c, key, m),
                Self::Customize(data) => data.key(c, key, m),
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_mode_key_dispatch.rs"]
mod tests;

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude mutation of the current pane mode during the query.
    pub(crate) unsafe fn mode_key_table(&self) -> Option<&'static core::ffi::CStr> {
        let owner = self.upgrade()?;
        unsafe {
            let mode = (owner.as_pane()).active_mode()?;
            mode.mode().key_table(mode)
        }
    }
    /// # Safety
    /// Exclude conflicting access to the current mode while updating its prefix.
    pub(crate) unsafe fn set_mode_prefix(&self, prefix: u_int) -> Option<bool> {
        let mut owner = self.upgrade()?;
        unsafe {
            let mode = (owner.as_pane_mut()).active_mode_mut().map(|mode| mode.into_entry())?;
            if !mode.mode().has_command() {
                return Some(false);
            }
            mode.prefix = prefix;
            Some(true)
        }
    }
    /// # Safety
    /// The current mode must have exclusive access to its state while dispatching.
    /// Dispatch may remove the mode; no pane owner is retained across that callback.
    pub(crate) unsafe fn mode_command(
        &self,
        client: Option<&mut ClientRef>,
        session: Option<&SessionRef>,
        link: Option<&crate::window::WinlinkRef>,
        args: &RustArguments,
        mouse: Option<&mut mouse_event>,
    ) -> bool {
        unsafe {
            let mut target = self.clone();
            let Some(mode) = target
                .get_mut()
                .and_then(|pane| pane.active_mode_mut().map(|mode| mode.into_entry()))
                .filter(|mode| mode.mode().has_command())
            else {
                return false;
            };
            mode.mode().command(
                mode,
                client.map(|c| c.as_client_mut()),
                session.map(|s| s.as_session()),
                link.and_then(|l| l.get()),
                args,
                mouse,
            );
        }
        true
    }
}

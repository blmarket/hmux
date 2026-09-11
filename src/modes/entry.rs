use crate::types::*;
use crate::screen::{RustScreen, Screen};
use crate::options::RustOptionsRef;

macro_rules! mode_entry {
    ($visibility:vis) => {
#[repr(C)]
pub struct window_mode_entry {
    $visibility wp: Option<RustWindowPaneWeak>,
    $visibility swp: Option<RustWindowPaneWeak>,
    $visibility state: WindowModeState,
    $visibility screen: Option<ModeScreen>,
    $visibility prefix: u_int,
}
    };
}
#[cfg(not(test))]
mode_entry!(pub(super));
#[cfg(test)]
mode_entry!(pub(crate));

impl window_mode_entry {
    /// Observes the pane while its owner still exists.
    pub(crate) fn pane_ref(&self) -> Option<RustWindowPaneWeak> {
        self.wp
            .as_ref()
            .filter(|pane| pane.is_alive())
            .cloned()
    }

    /// Observes the source pane directly until it is destroyed.
    pub(crate) fn source_pane_ref(&self) -> Option<crate::window::RustWindowPaneWeak> {
        self.swp
            .as_ref()
            .filter(|pane| pane.is_alive())
            .cloned()
    }
}


impl window_mode_entry {
    pub(crate) fn pending(pane: RustWindowPaneWeak, source: Option<RustWindowPaneWeak>) -> Self {
        Self { wp: Some(pane), swp: source, state: WindowModeState::None, screen: None, prefix: 1 }
    }
    pub(crate) unsafe fn initialize(&mut self, mode: WindowMode, pane: RustWindowPaneWeak,
        target: Option<&cmd_find_state>, args: Option<&crate::args::RustArguments>) {
        unsafe { mode.init(self, pane, target, args) };
    }
    pub(crate) unsafe fn release(&mut self) { unsafe { self.mode().free(self) }; }
    pub(crate) unsafe fn resize(&mut self, width: u32, height: u32) { unsafe { self.mode().resize(self, width, height) }; }
    pub(crate) fn shown_screen(&self) -> Option<ScreenBorrow<'_>> {
        match self.screen.as_ref()? {
            ModeScreen::Clock => {
                let WindowModeState::Clock(data) = &self.state else { return None };
                Some(ScreenBorrow::Owned(&data.screen))
            }
            ModeScreen::Shared(screen) => Some(ScreenBorrow::Shared(screen.borrow())),
        }
    }
    pub(crate) fn update_default_cursor(&mut self, options: &RustOptionsRef) {
        match self.screen.as_ref().expect("the shown mode has a screen") {
            ModeScreen::Clock => { self.state.clock().expect("clock mode has state").screen.set_default_cursor(options); }
            ModeScreen::Shared(screen) => screen.borrow_mut().set_default_cursor(options),
        }
    }
}

/// A scoped mode-engine context that cannot replace pane stack membership.
///
/// ```compile_fail
/// use tmux_c2rs::{WindowPane, types::window_mode_entry};
/// fn replace(pane: &mut dyn WindowPane, replacement: window_mode_entry) {
///     *pane.active_mode_mut().unwrap() = replacement;
/// }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::WindowPane;
/// fn payload(pane: &dyn WindowPane) {
///     let _ = &pane.active_mode().unwrap().state;
/// }
/// ```
pub struct ModeContext<'a>(&'a mut window_mode_entry);

pub(crate) enum ModeScreenTarget<'a> {
    Owned(crate::screen::ScreenMut<'a>),
    Shared(&'a ScreenRef),
}

impl<'a> ModeContext<'a> {
    pub(crate) fn new(entry: &'a mut window_mode_entry) -> Self { Self(entry) }
    #[cfg(not(test))]
    pub(super) fn into_entry(self) -> &'a mut window_mode_entry { self.0 }
    #[cfg(test)]
    pub(crate) fn into_entry(self) -> &'a mut window_mode_entry { self.0 }
    /// Refreshes mode rendering state through its implementation.
    /// # Safety
    /// Exclude conflicting mode access while its callback runs.
    pub unsafe fn style_changed(self) { unsafe { self.0.mode().style_changed(self.0) }; }
    pub(crate) fn screen_target(self) -> ModeScreenTarget<'a> {
        match self.0.screen.as_ref().expect("shown mode has a screen") {
            ModeScreen::Clock => ModeScreenTarget::Owned(crate::screen::ScreenMut::new(&mut self.0.state.clock().expect("clock mode has state").screen)),
            ModeScreen::Shared(screen) => ModeScreenTarget::Shared(screen),
        }
    }
}

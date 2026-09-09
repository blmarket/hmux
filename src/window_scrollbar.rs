//! Window-level scrollbar mode and position.

use core::ffi::c_int;

/// The cached scrollbar settings for a window.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowScrollbarSettings {
    pub mode: c_int,
    pub position: c_int,
}

/// Storage for window-level scrollbar settings.
pub trait WindowScrollbarState: Default {
    /// Returns the cached mode and position.
    fn scrollbar_settings(&self) -> WindowScrollbarSettings;

    /// Replaces the cached mode and position.
    fn set_scrollbar_settings(&mut self, settings: WindowScrollbarSettings);
}

/// The window scrollbar storage used by hmux.
#[derive(Default)]
pub struct RustWindowScrollbarState {
    settings: WindowScrollbarSettings,
}

impl WindowScrollbarState for RustWindowScrollbarState {
    fn scrollbar_settings(&self) -> WindowScrollbarSettings {
        self.settings
    }

    fn set_scrollbar_settings(&mut self, settings: WindowScrollbarSettings) {
        self.settings = settings;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_replaced_together() {
        let mut state = RustWindowScrollbarState::default();
        assert_eq!(state.scrollbar_settings(), Default::default());
        let settings = WindowScrollbarSettings {
            mode: 2,
            position: 1,
        };
        state.set_scrollbar_settings(settings);
        assert_eq!(state.scrollbar_settings(), settings);
    }
}

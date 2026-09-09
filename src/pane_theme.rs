//! The last terminal theme observed for a pane.

use crate::style::THEME_UNKNOWN;
use crate::types::client_theme;

/// Storage for a pane's last observed theme.
pub trait PaneThemeState {
    /// Returns the last observed theme.
    fn theme(&self) -> client_theme;

    /// Stores an observed theme.
    fn set_theme(&mut self, theme: client_theme);

    /// Stores a theme and returns whether it changed.
    fn replace(&mut self, theme: client_theme) -> bool;
}

/// The pane theme state used by hmux.
pub struct RustPaneThemeState {
    theme: client_theme,
}

impl Default for RustPaneThemeState {
    fn default() -> Self {
        Self {
            theme: THEME_UNKNOWN,
        }
    }
}

impl PaneThemeState for RustPaneThemeState {
    fn theme(&self) -> client_theme {
        self.theme
    }

    fn set_theme(&mut self, theme: client_theme) {
        self.theme = theme;
    }

    fn replace(&mut self, theme: client_theme) -> bool {
        let changed = self.theme != theme;
        self.theme = theme;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{THEME_DARK, THEME_LIGHT};

    #[test]
    fn replacement_reports_only_real_transitions() {
        let mut state = RustPaneThemeState::default();
        assert_eq!(state.theme(), THEME_UNKNOWN);
        assert!(!state.replace(THEME_UNKNOWN));
        assert!(state.replace(THEME_LIGHT));
        assert_eq!(state.theme(), THEME_LIGHT);
        assert!(!state.replace(THEME_LIGHT));
        state.set_theme(THEME_DARK);
        assert_eq!(state.theme(), THEME_DARK);
    }
}

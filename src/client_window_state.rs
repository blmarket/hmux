//! Stable access to per-client window state.

/// A client's window identifier and reported size.
pub trait ClientWindowState {
    /// Builds per-client state for a window.
    fn from_client_window_state(window_id: u32, width: u32, height: u32) -> Self
    where
        Self: Sized;

    /// Returns the window identifier.
    fn client_window_id(&self) -> u32;

    /// Returns the client-reported width and height.
    fn client_window_size(&self) -> (u32, u32);

    /// Sets the client-reported width and height.
    fn set_client_window_size(&mut self, width: u32, height: u32);
}

impl ClientWindowState for crate::types::client_window {
    fn from_client_window_state(window_id: u32, width: u32, height: u32) -> Self {
        Self {
            window: window_id,
            pane: None,
            sx: width,
            sy: height,
        }
    }
    fn client_window_id(&self) -> u32 {
        self.window
    }
    fn client_window_size(&self) -> (u32, u32) {
        (self.sx, self.sy)
    }
    fn set_client_window_size(&mut self, width: u32, height: u32) {
        self.sx = width;
        self.sy = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::client_window;

    #[test]
    fn state_round_trips_and_updates() {
        let mut state = client_window::from_client_window_state(7, 80, 24);
        assert_eq!(state.client_window_id(), 7);
        assert_eq!(state.client_window_size(), (80, 24));
        state.set_client_window_size(120, 40);
        assert_eq!(state.client_window_size(), (120, 40));
    }
}

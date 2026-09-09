//! Stable access to screen redraw geometry and display settings.

/// The status, pane decoration, and viewport state used for one screen redraw.
pub trait ScreenRedrawContext {
    /// Builds a redraw context from its portable state.
    fn from_screen_redraw_context(
        status: (u32, bool),
        pane_border: (i32, u32),
        pane_scrollbars: (i32, i32),
        viewport: (i32, i32, u32, u32),
    ) -> Self
    where
        Self: Sized;

    /// Returns the status line count.
    fn screen_redraw_status_lines(&self) -> u32;

    /// Returns whether the status line is above the window.
    fn screen_redraw_status_at_top(&self) -> bool;

    /// Sets the status line count and whether it is above the window.
    fn set_screen_redraw_status(&mut self, lines: u32, at_top: bool);

    /// Sets the status line count without changing its position.
    fn set_screen_redraw_status_lines(&mut self, lines: u32) {
        let at_top = self.screen_redraw_status_at_top();
        self.set_screen_redraw_status(lines, at_top);
    }

    /// Sets whether the status line is above the window without changing its count.
    fn set_screen_redraw_status_at_top(&mut self, at_top: bool) {
        let lines = self.screen_redraw_status_lines();
        self.set_screen_redraw_status(lines, at_top);
    }

    /// Returns the pane status position.
    fn screen_redraw_pane_status(&self) -> i32;

    /// Returns the pane border line style.
    fn screen_redraw_pane_lines(&self) -> u32;

    /// Sets the pane status position and border line style.
    fn set_screen_redraw_pane_border(&mut self, status: i32, lines: u32);

    /// Sets the pane status position without changing the border line style.
    fn set_screen_redraw_pane_status(&mut self, status: i32) {
        let lines = self.screen_redraw_pane_lines();
        self.set_screen_redraw_pane_border(status, lines);
    }

    /// Sets the pane border line style without changing the status position.
    fn set_screen_redraw_pane_lines(&mut self, lines: u32) {
        let status = self.screen_redraw_pane_status();
        self.set_screen_redraw_pane_border(status, lines);
    }

    /// Returns whether pane scrollbars are enabled.
    fn screen_redraw_pane_scrollbars_enabled(&self) -> i32;

    /// Returns the pane scrollbar position.
    fn screen_redraw_pane_scrollbars_position(&self) -> i32;

    /// Sets whether pane scrollbars are enabled and their position.
    fn set_screen_redraw_pane_scrollbars(&mut self, enabled: i32, position: i32);

    /// Returns the viewport x offset.
    fn screen_redraw_x(&self) -> i32;

    /// Returns the viewport y offset.
    fn screen_redraw_y(&self) -> i32;

    /// Returns the viewport width.
    fn screen_redraw_width(&self) -> u32;

    /// Returns the viewport height.
    fn screen_redraw_height(&self) -> u32;

    /// Sets the viewport as x, y, width, and height.
    fn set_screen_redraw_viewport(&mut self, x: i32, y: i32, width: u32, height: u32);

    /// Sets the viewport x offset without changing its other dimensions.
    fn set_screen_redraw_x(&mut self, x: i32) {
        let (_, y, width, height) = self.screen_redraw_viewport();
        self.set_screen_redraw_viewport(x, y, width, height);
    }

    /// Sets the viewport y offset without changing its other dimensions.
    fn set_screen_redraw_y(&mut self, y: i32) {
        let (x, _, width, height) = self.screen_redraw_viewport();
        self.set_screen_redraw_viewport(x, y, width, height);
    }

    /// Sets the viewport width without changing its other dimensions.
    fn set_screen_redraw_width(&mut self, width: u32) {
        let (x, y, _, height) = self.screen_redraw_viewport();
        self.set_screen_redraw_viewport(x, y, width, height);
    }

    /// Sets the viewport height without changing its other dimensions.
    fn set_screen_redraw_height(&mut self, height: u32) {
        let (x, y, width, _) = self.screen_redraw_viewport();
        self.set_screen_redraw_viewport(x, y, width, height);
    }

    /// Returns the status line count and whether it is above the window.
    fn screen_redraw_status(&self) -> (u32, bool) {
        (
            self.screen_redraw_status_lines(),
            self.screen_redraw_status_at_top(),
        )
    }

    /// Returns the pane status position and border line style.
    fn screen_redraw_pane_border(&self) -> (i32, u32) {
        (
            self.screen_redraw_pane_status(),
            self.screen_redraw_pane_lines(),
        )
    }

    /// Returns whether pane scrollbars are enabled and their position.
    fn screen_redraw_pane_scrollbars(&self) -> (i32, i32) {
        (
            self.screen_redraw_pane_scrollbars_enabled(),
            self.screen_redraw_pane_scrollbars_position(),
        )
    }

    /// Returns the viewport as x, y, width, and height.
    fn screen_redraw_viewport(&self) -> (i32, i32, u32, u32) {
        (
            self.screen_redraw_x(),
            self.screen_redraw_y(),
            self.screen_redraw_width(),
            self.screen_redraw_height(),
        )
    }
}

impl ScreenRedrawContext for crate::types::screen_redraw_ctx {
    fn from_screen_redraw_context(
        status: (u32, bool),
        pane_border: (i32, u32),
        pane_scrollbars: (i32, i32),
        viewport: (i32, i32, u32, u32),
    ) -> Self {
        Self {
            statuslines: status.0,
            statustop: i32::from(status.1),
            pane_status: pane_border.0,
            pane_lines: pane_border.1,
            pane_scrollbars: pane_scrollbars.0,
            pane_scrollbars_pos: pane_scrollbars.1,
            ox: viewport.0,
            oy: viewport.1,
            sx: viewport.2,
            sy: viewport.3,
            ..Self::default()
        }
    }
    fn screen_redraw_status_lines(&self) -> u32 {
        self.statuslines
    }
    fn screen_redraw_status_at_top(&self) -> bool {
        self.statustop != 0
    }
    fn set_screen_redraw_status(&mut self, lines: u32, at_top: bool) {
        self.statuslines = lines;
        self.statustop = i32::from(at_top);
    }
    fn screen_redraw_pane_status(&self) -> i32 {
        self.pane_status
    }
    fn screen_redraw_pane_lines(&self) -> u32 {
        self.pane_lines
    }
    fn set_screen_redraw_pane_border(&mut self, status: i32, lines: u32) {
        self.pane_status = status;
        self.pane_lines = lines;
    }
    fn screen_redraw_pane_scrollbars_enabled(&self) -> i32 {
        self.pane_scrollbars
    }
    fn screen_redraw_pane_scrollbars_position(&self) -> i32 {
        self.pane_scrollbars_pos
    }
    fn set_screen_redraw_pane_scrollbars(&mut self, enabled: i32, position: i32) {
        self.pane_scrollbars = enabled;
        self.pane_scrollbars_pos = position;
    }
    fn screen_redraw_x(&self) -> i32 {
        self.ox
    }
    fn screen_redraw_y(&self) -> i32 {
        self.oy
    }
    fn screen_redraw_width(&self) -> u32 {
        self.sx
    }
    fn screen_redraw_height(&self) -> u32 {
        self.sy
    }
    fn set_screen_redraw_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.ox = x;
        self.oy = y;
        self.sx = width;
        self.sy = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::screen_redraw_ctx;

    #[test]
    fn redraw_state_round_trips_and_updates() {
        let mut context = screen_redraw_ctx::from_screen_redraw_context(
            (2, true),
            (1, 3),
            (1, 2),
            (-4, 5, 80, 24),
        );
        assert_eq!(context.screen_redraw_status(), (2, true));
        assert_eq!(context.screen_redraw_pane_border(), (1, 3));
        assert_eq!(context.screen_redraw_pane_scrollbars(), (1, 2));
        assert_eq!(context.screen_redraw_viewport(), (-4, 5, 80, 24));

        context.set_screen_redraw_status(1, false);
        context.set_screen_redraw_pane_border(2, 4);
        context.set_screen_redraw_pane_scrollbars(0, -1);
        context.set_screen_redraw_viewport(6, -7, 120, 40);
        assert_eq!(context.screen_redraw_status(), (1, false));
        assert_eq!(context.screen_redraw_pane_border(), (2, 4));
        assert_eq!(context.screen_redraw_pane_scrollbars(), (0, -1));
        assert_eq!(context.screen_redraw_viewport(), (6, -7, 120, 40));
    }
}

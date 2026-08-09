//! The in-house engine as a screen the server can use.
//!
//! This is the other side of [`crate::screen::VtScreen`] from
//! [`crate::ghostty`]: same trait, same types, no libghostty-vt. It is the
//! backend [`crate::PaneScreen`] names, so this is what a pane's grid
//! actually is; the Ghostty one survives behind its feature as the thing the
//! differential harness diffs against.

use std::io;
use std::sync::Arc;

use super::dispatch::Engine;
use super::dump;
use super::keys;
use super::screen::DEFAULT_HISTORY_LIMIT;
use crate::input::{InputEncoder, MouseEvent};
use crate::parser::Token;
use crate::screen::{CaptureExtent, Grid, GridDims, ScreenImage, ScreenOptions, VtScreen};

/// hmux's own screen.
pub struct EngineScreen {
    engine: Engine,
}

impl EngineScreen {
    pub fn new(cols: u16, rows: u16) -> EngineScreen {
        EngineScreen {
            engine: Engine::new(
                usize::from(cols.max(1)),
                usize::from(rows.max(1)),
                DEFAULT_HISTORY_LIMIT,
            ),
        }
    }
}

impl VtScreen for EngineScreen {
    fn apply(&mut self, token: &Token) {
        self.engine.apply(&token.kind);
    }

    fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        self.engine
            .screen
            .resize(usize::from(cols.max(1)), usize::from(rows.max(1)));
        Ok(())
    }

    fn set_options(&mut self, options: ScreenOptions) {
        self.engine.screen.options = options;
    }

    fn set_history_limit(&mut self, limit: usize) {
        self.engine.screen.set_history_limit(limit);
    }

    fn modes(&self) -> u32 {
        self.engine.screen.mode
    }

    fn trim_history_below_cursor(&mut self) -> io::Result<()> {
        let screen = &mut self.engine.screen;
        // The rows below the cursor are what goes, and there is only as much
        // history as there is to pull up in their place.
        let adjust = (screen.sy() - 1 - screen.cy).min(screen.grid.hsize);
        screen.grid.remove_history(adjust);
        screen.cy += adjust;
        Ok(())
    }

    fn cursor_position(&self) -> io::Result<(u16, u16)> {
        let screen = &self.engine.screen;
        Ok((
            u16::try_from(screen.cx).unwrap_or(u16::MAX),
            u16::try_from(screen.cy).unwrap_or(u16::MAX),
        ))
    }

    fn cursor_visible(&self) -> io::Result<bool> {
        Ok(dump::cursor_visible(&self.engine.screen))
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        Ok(self.engine.screen.grid.hsize)
    }

    fn grid_dims(&self) -> io::Result<GridDims> {
        let grid = &self.engine.screen.grid;
        Ok(GridDims {
            cols: u16::try_from(grid.sx).unwrap_or(u16::MAX),
            viewport_rows: u16::try_from(grid.sy).unwrap_or(u16::MAX),
            scrollback_rows: grid.hsize,
            total_rows: grid.total(),
        })
    }

    fn grid_snapshot(&self) -> io::Result<Grid> {
        let total = self.engine.screen.grid.total();
        Ok(dump::snapshot(&self.engine.screen, 0, total))
    }

    fn images(&self) -> Vec<ScreenImage> {
        let images = &self.engine.screen.images;
        let revision = images.revision();
        images
            .live()
            .map(|image| ScreenImage {
                px: u16::try_from(image.px).unwrap_or(u16::MAX),
                py: u16::try_from(image.py).unwrap_or(u16::MAX),
                sx: u16::try_from(image.sx).unwrap_or(u16::MAX),
                sy: u16::try_from(image.sy).unwrap_or(u16::MAX),
                revision,
                data: Arc::clone(&image.data),
                fallback: Arc::clone(&image.fallback),
            })
            .collect()
    }

    fn inactive_snapshot(&self) -> io::Result<Option<(Grid, Vec<u8>)>> {
        let screen = &self.engine.screen;
        Ok(screen.saved_grid().map(|grid| {
            let total = grid.total();
            (
                dump::snapshot_grid(screen, grid, 0, total),
                // `-a` reads the whole displaced screen; which extent the
                // capture wants is decided when its rows are serialized.
                dump::vt_grid(
                    screen,
                    grid,
                    0,
                    total,
                    dump::RowExtent::Capture(CaptureExtent::Allocated),
                ),
            )
        }))
    }

    fn grid_snapshot_range(&self, start: usize, count: usize) -> io::Result<Grid> {
        Ok(dump::snapshot(&self.engine.screen, start, count))
    }

    fn dump_plain(&self) -> io::Result<String> {
        let total = self.engine.screen.grid.total();
        Ok(dump::plain(&self.engine.screen, 0, total, false))
    }

    fn dump_plain_unwrapped(&self) -> io::Result<String> {
        let total = self.engine.screen.grid.total();
        Ok(dump::plain(&self.engine.screen, 0, total, true))
    }

    fn dump_plain_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<String> {
        if rows == 0 || cols == 0 {
            return Ok(String::new());
        }
        Ok(dump::plain(&self.engine.screen, start, rows, false))
    }

    fn dump_vt(&self) -> io::Result<Vec<u8>> {
        let total = self.engine.screen.grid.total();
        Ok(dump::vt(
            &self.engine.screen,
            0,
            total,
            dump::RowExtent::Redraw,
        ))
    }

    fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<Vec<u8>> {
        if rows == 0 || cols == 0 {
            return Ok(Vec::new());
        }
        Ok(dump::vt(
            &self.engine.screen,
            start,
            rows,
            dump::RowExtent::Redraw,
        ))
    }

    fn dump_vt_capture_rows(
        &self,
        start: usize,
        rows: usize,
        cols: u16,
        extent: CaptureExtent,
    ) -> io::Result<Vec<u8>> {
        if rows == 0 || cols == 0 {
            return Ok(Vec::new());
        }
        Ok(dump::vt(
            &self.engine.screen,
            start,
            rows,
            dump::RowExtent::Capture(extent),
        ))
    }
}

impl InputEncoder for EngineScreen {
    fn encode_mouse(&self, mouse: MouseEvent) -> io::Result<Vec<u8>> {
        Ok(keys::encode_mouse(&self.engine.screen, mouse))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    fn screen(cols: u16, rows: u16, input: &[u8]) -> EngineScreen {
        let mut screen = EngineScreen::new(cols, rows);
        for token in tokenize(input) {
            screen.apply(&token);
        }
        screen
    }

    #[test]
    fn the_engine_answers_the_whole_seam() {
        let screen = screen(10, 3, b"hello\r\nworld");
        assert_eq!(screen.cursor_position().expect("cursor"), (5, 1));
        assert!(screen.cursor_visible().expect("cursor mode"));
        assert_eq!(screen.scrollback_rows().expect("history"), 0);
        let dims = screen.grid_dims().expect("dims");
        assert_eq!((dims.cols, dims.viewport_rows, dims.total_rows), (10, 3, 3));
        assert!(screen.dump_plain().expect("plain").contains("world"));
        assert!(!screen.dump_vt().expect("vt").is_empty());
        assert_eq!(screen.grid_snapshot().expect("snapshot").rows.len(), 3);
    }

    #[test]
    fn a_hidden_cursor_is_reported_hidden() {
        let screen = screen(10, 2, b"\x1b[?25l");
        assert!(!screen.cursor_visible().expect("cursor mode"));
    }

    #[test]
    fn a_range_snapshot_starts_at_the_row_it_was_asked_for() {
        let screen = screen(10, 2, b"a\r\nb\r\nc\r\nd");
        assert_eq!(screen.scrollback_rows().expect("history"), 2);
        let grid = screen.grid_snapshot_range(2, 2).expect("range");
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].cells[0].text, "c");
        assert_eq!(
            grid.scrollback_rows, 2,
            "the dimensions still describe the whole grid"
        );
    }

    #[test]
    fn resizing_rewraps_the_content_and_keeps_the_cursor_with_it() {
        let mut screen = screen(10, 3, b"abcdefgh");
        screen.resize(4, 2).expect("resize");
        let dims = screen.grid_dims().expect("dims");
        assert_eq!((dims.cols, dims.viewport_rows), (4, 2));
        assert!(screen.dump_plain().expect("plain").contains("abcd\nefgh"));
        let (cx, cy) = screen.cursor_position().expect("cursor");
        // Four is the pending-wrap column of a four-column screen, which is
        // where tmux parks a cursor that has just filled the last one.
        assert_eq!((cx, cy), (4, 0));
    }
}

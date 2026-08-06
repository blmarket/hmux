//! The in-house engine as a screen the server can use.
//!
//! This is the other side of [`crate::vt::screen::VtScreen`] from
//! [`crate::vt::ghostty`]: same trait, same types, no libghostty-vt. It is not
//! the default backend — [`crate::vt::PaneScreen`] still names the Ghostty one
//! — and making it the default is the signed-off decision the plan calls
//! Phase 3.

use std::io;

use super::dispatch::Engine;
use super::dump;
use super::keys;
use super::screen::DEFAULT_HISTORY_LIMIT;
use crate::vt::input::{InputEncoder, KeyEvent, MouseEvent};
use crate::vt::parser::Token;
use crate::vt::screen::{Grid, GridDims, VtScreen};

/// hmux's own screen.
pub(crate) struct EngineScreen {
    engine: Engine,
}

impl EngineScreen {
    pub(crate) fn new(cols: u16, rows: u16) -> EngineScreen {
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
        Ok(dump::vt(&self.engine.screen, 0, total))
    }

    fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<Vec<u8>> {
        if rows == 0 || cols == 0 {
            return Ok(Vec::new());
        }
        Ok(dump::vt(&self.engine.screen, start, rows))
    }
}

impl InputEncoder for EngineScreen {
    fn encode_key(&self, key: KeyEvent<'_>) -> io::Result<Vec<u8>> {
        Ok(keys::encode_key(&self.engine.screen, key))
    }

    fn encode_mouse(&self, mouse: MouseEvent) -> io::Result<Vec<u8>> {
        Ok(keys::encode_mouse(&self.engine.screen, mouse))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vt::parser::tokenize;

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

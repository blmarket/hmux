//! A terminal model for out-of-process test harnesses.
//!
//! The conformance suite drives a real client tty and then has to say what the
//! screen should look like. It computes that with the same emulation the daemon
//! used to produce the bytes, so a difference in the assertion is a difference
//! in the daemon, not in two independent readings of a byte stream.
//!
//! This is the only public window onto the emulation seam, and it is
//! deliberately narrow: feed bytes, read the screen back. Nothing here is an
//! end-user contract — the daemon's contract is its tmux-compatible command
//! line and wire protocol.

use std::io;

use crate::vt::parser::Parser;
use crate::vt::screen::VtScreen;
use crate::vt::PaneScreen;

/// A screen a harness can feed bytes to and read back.
pub struct TerminalModel {
    parser: Parser,
    screen: PaneScreen,
}

impl TerminalModel {
    /// A `cols`×`rows` model with the daemon's own scrollback settings.
    pub fn new(cols: u16, rows: u16) -> io::Result<TerminalModel> {
        Ok(TerminalModel {
            parser: Parser::default(),
            screen: PaneScreen::new(cols, rows),
        })
    }

    /// Apply a chunk of terminal bytes. A sequence split across calls is
    /// retained, as it would be across a pane's pty reads.
    pub fn write(&mut self, data: &[u8]) {
        let screen = &mut self.screen;
        self.parser.parse(data, |token| screen.apply(&token));
    }

    /// Resize the model, as a client resize would resize the pane.
    pub fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        self.screen.resize(cols, rows)
    }

    /// The whole screen as plain text, trailing whitespace trimmed.
    pub fn dump_plain(&self) -> io::Result<String> {
        self.screen.dump_plain()
    }

    /// The whole screen as VT escape sequences, as the compositor would write
    /// them to a client tty.
    pub fn dump_vt(&self) -> io::Result<Vec<u8>> {
        self.screen.dump_vt()
    }

    /// How many rows of history sit above the viewport.
    pub fn scrollback_rows(&self) -> io::Result<usize> {
        self.screen.scrollback_rows()
    }
}

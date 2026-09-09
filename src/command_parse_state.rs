//! Portable state carried while parsing commands.

use core::ffi::{CStr, c_int};

/// Observable cursor, condition, and error state of a command parser.
pub trait CommandParseState: Default {
    /// Whether the parser is reading an owned file buffer.
    fn command_parse_has_file(&self) -> bool;

    /// Whether the parser is reading a borrowed input buffer.
    fn command_parse_has_buffer(&self) -> bool;

    /// Whether the parser has an active parse input.
    fn command_parse_has_input(&self) -> bool;

    /// Returns the length and current offset of the input buffer.
    fn command_parse_position(&self) -> (usize, usize);

    /// Replaces the current input offset.
    fn set_command_parse_offset(&mut self, offset: usize);

    /// Returns the active conditional state.
    fn command_parse_condition(&self) -> c_int;

    /// Replaces the active conditional state.
    fn set_command_parse_condition(&mut self, condition: c_int);

    /// Returns whether an end of line has been observed.
    fn command_parse_eol(&self) -> c_int;

    /// Replaces the end-of-line state.
    fn set_command_parse_eol(&mut self, eol: c_int);

    /// Returns whether end of input has been observed.
    fn command_parse_eof(&self) -> c_int;

    /// Replaces the end-of-input state.
    fn set_command_parse_eof(&mut self, eof: c_int);

    /// Returns the number of active escape levels.
    fn command_parse_escapes(&self) -> u32;

    /// Replaces the number of active escape levels.
    fn set_command_parse_escapes(&mut self, escapes: u32);

    /// Returns the parser error, when one has been recorded.
    fn command_parse_error(&self) -> Option<&CStr>;

    /// Replaces or clears the parser error.
    fn set_command_parse_error(&mut self, error: Option<&CStr>);
}

impl CommandParseState for crate::cmd::cmd_parse_state<'_> {
    fn command_parse_has_file(&self) -> bool {
        self.f.is_some()
    }
    fn command_parse_has_buffer(&self) -> bool {
        self.buf.is_some()
    }
    fn command_parse_has_input(&self) -> bool {
        self.input.is_some()
    }
    fn command_parse_position(&self) -> (usize, usize) {
        (self.len, self.off)
    }
    fn set_command_parse_offset(&mut self, offset: usize) {
        self.off = offset;
    }
    fn command_parse_condition(&self) -> c_int {
        self.condition
    }
    fn set_command_parse_condition(&mut self, condition: c_int) {
        self.condition = condition;
    }
    fn command_parse_eol(&self) -> c_int {
        self.eol
    }
    fn set_command_parse_eol(&mut self, eol: c_int) {
        self.eol = eol;
    }
    fn command_parse_eof(&self) -> c_int {
        self.eof
    }
    fn set_command_parse_eof(&mut self, eof: c_int) {
        self.eof = eof;
    }
    fn command_parse_escapes(&self) -> u32 {
        self.escapes
    }
    fn set_command_parse_escapes(&mut self, escapes: u32) {
        self.escapes = escapes;
    }
    fn command_parse_error(&self) -> Option<&CStr> {
        self.error.as_deref()
    }
    fn set_command_parse_error(&mut self, error: Option<&CStr>) {
        self.error = error.map(CStr::to_owned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::cmd_parse_state;

    #[test]
    fn parser_state_defaults_and_updates() {
        let mut state = cmd_parse_state::default();
        assert!(!state.command_parse_has_file());
        assert!(!state.command_parse_has_buffer());
        assert!(!state.command_parse_has_input());
        assert_eq!(state.command_parse_position(), (0, 0));
        assert_eq!(state.command_parse_error(), None);

        state.set_command_parse_offset(7);
        state.set_command_parse_condition(1);
        state.set_command_parse_eol(2);
        state.set_command_parse_eof(3);
        state.set_command_parse_escapes(4);
        state.set_command_parse_error(Some(c"syntax error"));

        assert_eq!(state.command_parse_position(), (0, 7));
        assert_eq!(state.command_parse_condition(), 1);
        assert_eq!(state.command_parse_eol(), 2);
        assert_eq!(state.command_parse_eof(), 3);
        assert_eq!(state.command_parse_escapes(), 4);
        assert_eq!(state.command_parse_error(), Some(c"syntax error"));
    }
}

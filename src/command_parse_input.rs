//! Stable access to command parser context.

use crate::cmdq::CmdqItemRef;
use crate::types::{ClientRef, cmd_find_state, u_int};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// Source, queue, client, and target context used while parsing commands.
pub trait CommandParseInput {
    /// Returns the parser behavior flags.
    fn command_parse_flags(&self) -> c_int;

    /// Replaces the parser behavior flags.
    fn set_command_parse_flags(&mut self, flags: c_int);

    /// Adds parser behavior flags.
    fn add_command_parse_flags(&mut self, flags: c_int) {
        self.set_command_parse_flags(self.command_parse_flags() | flags);
    }

    /// Removes parser behavior flags.
    fn remove_command_parse_flags(&mut self, flags: c_int) {
        self.set_command_parse_flags(self.command_parse_flags() & !flags);
    }

    /// Returns the source filename, when parsing came from a file.
    fn command_parse_file(&self) -> Option<&CStr>;

    /// Replaces the source filename.
    fn set_command_parse_file(&mut self, file: Option<CString>);

    /// Returns the current source line.
    fn command_parse_line(&self) -> u_int;

    /// Replaces the current source line.
    fn set_command_parse_line(&mut self, line: u_int);

    /// Advances the current source line with C-compatible wrapping.
    fn advance_command_parse_line(&mut self) {
        self.set_command_parse_line(self.command_parse_line().wrapping_add(1));
    }

    /// Returns the queue item associated with the parse when it still exists.
    fn command_parse_item(&self) -> Option<CmdqItemRef>;

    /// Replaces the queue item observed by the parse.
    fn set_command_parse_item(&mut self, item: Option<CmdqItemRef>);

    /// Returns the client associated with the parse when it still exists.
    fn command_parse_client(&self) -> Option<ClientRef>;

    /// Replaces the client observed by the parse.
    fn set_command_parse_client(&mut self, client: Option<ClientRef>);

    /// Returns the target state associated with the parse.
    fn command_parse_find_state(&self) -> &cmd_find_state;

    /// Returns mutable access to the target state associated with the parse.
    fn command_parse_find_state_mut(&mut self) -> &mut cmd_find_state;
}

impl CommandParseInput for crate::types::cmd_parse_input {
    fn command_parse_flags(&self) -> c_int {
        self.flags
    }

    fn set_command_parse_flags(&mut self, flags: c_int) {
        self.flags = flags;
    }

    fn command_parse_file(&self) -> Option<&CStr> {
        self.file()
    }

    fn set_command_parse_file(&mut self, file: Option<CString>) {
        self.file = file;
    }

    fn command_parse_line(&self) -> u_int {
        self.line
    }

    fn set_command_parse_line(&mut self, line: u_int) {
        self.line = line;
    }

    fn command_parse_item(&self) -> Option<CmdqItemRef> {
        self.item
            .as_ref()
            .and_then(crate::cmdq::CmdqItemWeak::upgrade)
    }

    fn set_command_parse_item(&mut self, item: Option<CmdqItemRef>) {
        self.item = item.map(|item| item.downgrade());
    }

    fn command_parse_client(&self) -> Option<ClientRef> {
        self.client()
    }

    fn set_command_parse_client(&mut self, client: Option<ClientRef>) {
        self.c = client.map(|client| client.downgrade());
    }

    fn command_parse_find_state(&self) -> &cmd_find_state {
        &self.fs
    }

    fn command_parse_find_state_mut(&mut self) -> &mut cmd_find_state {
        &mut self.fs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::cmd_parse_input;

    #[test]
    fn source_and_flags_are_replaced_through_the_contract() {
        let mut input = cmd_parse_input::default();
        input.set_command_parse_flags(1);
        input.add_command_parse_flags(4);
        input.remove_command_parse_flags(1);
        input.set_command_parse_file(Some(c"input.conf".to_owned()));
        input.set_command_parse_line(9);

        assert_eq!(input.command_parse_flags(), 4);
        assert_eq!(input.command_parse_file(), Some(c"input.conf"));
        assert_eq!(input.command_parse_line(), 9);
        assert!(input.command_parse_item().is_none());
        assert!(input.command_parse_client().is_none());
    }
}

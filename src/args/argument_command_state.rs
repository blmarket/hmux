//! Stable access to a command prepared from parsed arguments.

use crate::cmd::CmdListRef;
use crate::types::{ClientRef, cmd_parse_input, u_int};
use core::ffi::CStr;
use std::ffi::CString;

/// The command payload and parse context retained for deferred execution.
pub trait ArgumentCommandState: Default {
    /// Returns the already-parsed command list, when the argument supplied one.
    fn prepared_command_list(&self) -> Option<&CmdListRef>;

    /// Replaces the already-parsed command list.
    fn set_prepared_command_list(&mut self, command_list: Option<CmdListRef>);

    /// Removes and returns the already-parsed command list.
    fn take_prepared_command_list(&mut self) -> Option<CmdListRef>;

    /// Returns the command template retained for later parsing.
    fn prepared_command_text(&self) -> Option<&CStr>;

    /// Replaces the command template retained for later parsing.
    fn set_prepared_command_text(&mut self, command: Option<CString>);

    /// Returns the context used when the command template is parsed.
    fn prepared_command_parse_input(&self) -> &cmd_parse_input;

    /// Returns mutable access to the context used during parsing.
    fn prepared_command_parse_input_mut(&mut self) -> &mut cmd_parse_input;

    /// Sets the command source and source line together.
    fn set_prepared_command_source(&mut self, file: Option<&CStr>, line: u_int);

    /// Holds the parse client in the command's parse context.
    fn set_prepared_command_client(&mut self, client: Option<ClientRef>);

    /// Returns the client held for parsing, when it is still present.
    fn prepared_command_client(&self) -> Option<&ClientRef>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandParseInput;
    use crate::args::args_command_state;

    #[test]
    fn text_and_source_are_replaced_through_the_contract() {
        let mut state = args_command_state::default();
        state.set_prepared_command_text(Some(c"display-message one".to_owned()));
        state.set_prepared_command_source(Some(c"one.conf"), 4);
        state.set_prepared_command_text(Some(c"list-sessions".to_owned()));
        state.set_prepared_command_source(Some(c"two.conf"), 9);

        assert_eq!(state.prepared_command_text(), Some(c"list-sessions"));
        assert_eq!(
            state.prepared_command_parse_input().command_parse_file(),
            Some(c"two.conf")
        );
        assert_eq!(state.prepared_command_parse_input().command_parse_line(), 9);
        assert!(state.prepared_command_list().is_none());
        assert!(state.prepared_command_client().is_none());
    }

    #[test]
    fn prepared_parse_context_retains_and_releases_its_client() {
        let client = ClientRef::new(crate::types::client::default());
        let weak = client.downgrade();
        let mut state = args_command_state::default();
        state.set_prepared_command_client(Some(client));
        let mut context = state.prepared_command_parse_input().clone();
        assert!(context.command_parse_client().is_some());
        state.set_prepared_command_client(None);
        assert!(state.prepared_command_client().is_none());
        assert!(state.prepared_command_parse_input().command_parse_client().is_none());
        assert!(weak.upgrade().is_some());
        context.set_command_parse_client(None);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn ordinary_parse_context_does_not_retain_its_client() {
        let client = ClientRef::new(crate::types::client::default());
        let weak = client.downgrade();
        let mut context = cmd_parse_input::default();
        context.set_command_parse_client(Some(client));
        assert!(weak.upgrade().is_none());
        assert!(context.command_parse_client().is_none());
    }

    #[test]
    fn parsed_command_lists_can_be_replaced_and_taken() {
        let mut state = args_command_state::default();
        state.set_prepared_command_list(Some(CmdListRef::empty()));
        assert!(state.prepared_command_list().is_some());
        assert!(state.take_prepared_command_list().is_some());
        assert!(state.prepared_command_list().is_none());
    }
}

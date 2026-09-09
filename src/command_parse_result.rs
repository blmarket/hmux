//! Stable access to the result of parsing commands.

use crate::cmd::CmdListRef;
use crate::types::cmd_parse_status;
use core::ffi::CStr;
use std::ffi::CString;

/// The status and owned outputs produced by a command parse.
pub trait CommandParseResult: Default {
    /// Returns the parse status.
    fn command_parse_status(&self) -> cmd_parse_status;

    /// Replaces the parse status.
    fn set_command_parse_status(&mut self, status: cmd_parse_status);

    /// Returns the parsed command list, when parsing succeeded with one.
    fn command_parse_list(&self) -> Option<&CmdListRef>;

    /// Replaces the parsed command list.
    fn set_command_parse_list(&mut self, command_list: Option<CmdListRef>);

    /// Removes and returns the parsed command list.
    fn take_command_parse_list(&mut self) -> Option<CmdListRef>;

    /// Returns the parse error, when parsing failed with one.
    fn command_parse_error(&self) -> Option<&CStr>;

    /// Replaces the parse error.
    fn set_command_parse_error(&mut self, error: Option<CString>);

    /// Removes and returns the parse error.
    fn take_command_parse_error(&mut self) -> Option<CString>;
}

impl CommandParseResult for crate::types::cmd_parse_result {
    fn command_parse_status(&self) -> cmd_parse_status {
        self.status
    }
    fn set_command_parse_status(&mut self, status: cmd_parse_status) {
        self.status = status;
    }
    fn command_parse_list(&self) -> Option<&CmdListRef> {
        self.cmdlist.as_ref()
    }
    fn set_command_parse_list(&mut self, command_list: Option<CmdListRef>) {
        self.cmdlist = command_list;
    }
    fn take_command_parse_list(&mut self) -> Option<CmdListRef> {
        self.cmdlist.take()
    }
    fn command_parse_error(&self) -> Option<&CStr> {
        self.error.as_deref()
    }
    fn set_command_parse_error(&mut self, error: Option<CString>) {
        self.error = error;
    }
    fn take_command_parse_error(&mut self) -> Option<CString> {
        self.error.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::cmd_parse_result;

    #[test]
    fn result_outputs_are_replaced_and_taken() {
        let mut result = cmd_parse_result::default();
        result.set_command_parse_status(1);
        result.set_command_parse_error(Some(c"first".to_owned()));
        result.set_command_parse_error(Some(c"second".to_owned()));
        result.set_command_parse_list(Some(CmdListRef::empty()));

        assert_eq!(result.command_parse_status(), 1);
        assert_eq!(result.command_parse_error(), Some(c"second"));
        assert!(result.command_parse_list().is_some());
        assert_eq!(
            result.take_command_parse_error().as_deref(),
            Some(c"second")
        );
        assert!(result.take_command_parse_list().is_some());
        assert!(result.command_parse_error().is_none());
        assert!(result.command_parse_list().is_none());
    }
}

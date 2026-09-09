//! Command-language parsing boundary.

use crate::cmd::{
    cmd_parse_from_arguments_impl, cmd_parse_from_buffer_impl, cmd_parse_from_file_impl,
    cmd_parse_from_string_impl,
};
use crate::types::{ArgsValue, cmd_parse_input, cmd_parse_result};
use core::ffi::CStr;

/// Parses command language into executable command lists.
pub trait CommandParser {
    /// Parser context carried across an input source.
    type Input: Default;
    /// Result returned by this parser implementation.
    type Result;
    /// One already-separated command argument.
    type Argument;

    /// Parses an owned file buffer.
    ///
    /// # Safety
    ///
    /// References retained by `input` must remain valid for the parse and any
    /// command list it produces.
    unsafe fn parse_file(&self, file: Vec<u8>, input: Option<&mut Self::Input>) -> Self::Result;

    /// Parses a borrowed byte buffer.
    ///
    /// # Safety
    ///
    /// References retained by `input` must remain valid for the parse and any
    /// command list it produces.
    unsafe fn parse_buffer(&self, buffer: &[u8], input: Option<&mut Self::Input>) -> Self::Result;

    /// Parses one command string as a single command group.
    ///
    /// # Safety
    ///
    /// References retained by `input` must remain valid for the parse and any
    /// command list it produces.
    unsafe fn parse_string(&self, string: &CStr, input: Option<&mut Self::Input>) -> Self::Result;

    /// Parses arguments already separated by the protocol client.
    ///
    /// # Safety
    ///
    /// References retained by `input` and nested command arguments must remain
    /// valid for the parse and any command list it produces.
    unsafe fn parse_arguments(
        &self,
        arguments: &[Self::Argument],
        input: Option<&mut Self::Input>,
    ) -> Self::Result;
}

/// Command-language parser implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustCommandParser;

impl CommandParser for RustCommandParser {
    type Input = cmd_parse_input;
    type Result = cmd_parse_result;
    type Argument = ArgsValue;

    unsafe fn parse_file(&self, file: Vec<u8>, input: Option<&mut Self::Input>) -> Self::Result {
        unsafe { cmd_parse_from_file_impl(file, input) }
    }

    unsafe fn parse_buffer(&self, buffer: &[u8], input: Option<&mut Self::Input>) -> Self::Result {
        unsafe { cmd_parse_from_buffer_impl(buffer, input) }
    }

    unsafe fn parse_string(&self, string: &CStr, input: Option<&mut Self::Input>) -> Self::Result {
        unsafe { cmd_parse_from_string_impl(string, input) }
    }

    unsafe fn parse_arguments(
        &self,
        arguments: &[Self::Argument],
        input: Option<&mut Self::Input>,
    ) -> Self::Result {
        unsafe { cmd_parse_from_arguments_impl(arguments, input) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_fixtures::globals;
    use crate::types::ArgsValue;
    use crate::{CommandList, CommandParseResult};

    fn succeeds(result: &cmd_parse_result, count: usize) {
        assert_eq!(result.command_parse_status(), 1);
        assert_eq!(
            result
                .command_parse_list()
                .unwrap()
                .with(CommandList::command_count),
            count
        );
        assert!(result.command_parse_error().is_none());
    }

    #[test]
    fn every_input_form_crosses_the_parser_boundary() {
        let _guard = globals();
        unsafe {
            let parser = RustCommandParser;
            succeeds(&parser.parse_string(c"list-buffers", None), 1);
            succeeds(
                &parser.parse_buffer(b"list-buffers; list-sessions", None),
                2,
            );
            succeeds(
                &parser.parse_file(b"list-buffers\nlist-sessions\n".to_vec(), None),
                2,
            );
            let arguments = [ArgsValue::String(c"list-buffers".to_owned())];
            succeeds(&parser.parse_arguments(&arguments, None), 1);
        }
    }

    #[test]
    fn syntax_errors_cross_the_parser_boundary() {
        let _guard = globals();
        let result = unsafe { RustCommandParser.parse_string(c"no-such-command", None) };
        assert_eq!(result.command_parse_status(), 0);
        assert!(result.command_parse_list().is_none());
        assert_eq!(
            result.command_parse_error(),
            Some(c"unknown command: no-such-command")
        );
    }
}

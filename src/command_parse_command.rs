//! Portable state for one command in a parser tree.

use core::ffi::{CStr, c_uint};

/// Source identity and ordered string arguments of one parsed command.
pub trait CommandParseCommand {
    /// Builds a command at `line` with no arguments.
    fn from_command_parse_command(line: c_uint) -> Self
    where
        Self: Sized;

    /// Returns the source line of the command.
    fn command_parse_command_line(&self) -> c_uint;

    /// Replaces the source line of the command.
    fn set_command_parse_command_line(&mut self, line: c_uint);

    /// Returns the number of arguments in parser order.
    fn command_parse_command_argument_count(&self) -> usize;

    /// Returns the raw kind of the argument at `index`.
    fn command_parse_command_argument_kind(&self, index: usize) -> Option<c_uint>;

    /// Returns the string payload of the argument at `index`, when present.
    fn command_parse_command_argument_string(&self, index: usize) -> Option<&CStr>;

    /// Appends a string argument.
    fn push_command_parse_string(&mut self, string: &CStr);

    /// Removes every argument.
    fn clear_command_parse_arguments(&mut self);
}

impl CommandParseCommand for crate::cmd::cmd_parse_command {
    fn from_command_parse_command(line: c_uint) -> Self {
        Self {
            line,
            arguments: Vec::new(),
        }
    }

    fn command_parse_command_line(&self) -> c_uint {
        self.line
    }

    fn set_command_parse_command_line(&mut self, line: c_uint) {
        self.line = line;
    }

    fn command_parse_command_argument_count(&self) -> usize {
        self.arguments.len()
    }

    fn command_parse_command_argument_kind(&self, index: usize) -> Option<c_uint> {
        self.arguments.get(index).map(|argument| argument.type_0)
    }

    fn command_parse_command_argument_string(&self, index: usize) -> Option<&CStr> {
        self.arguments.get(index)?.string.as_deref()
    }

    fn push_command_parse_string(&mut self, string: &CStr) {
        self.arguments
            .push(Box::new(crate::cmd::cmd_parse_argument {
                type_0: crate::cmd::CMD_PARSE_STRING,
                string: Some(string.to_owned()),
                commands: None,
                cmdlist: None,
            }));
    }

    fn clear_command_parse_arguments(&mut self) {
        self.arguments.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{CMD_PARSE_STRING, cmd_parse_command};

    #[test]
    fn command_line_and_arguments_round_trip() {
        let mut command = cmd_parse_command::from_command_parse_command(12);
        assert_eq!(command.command_parse_command_line(), 12);
        assert_eq!(command.command_parse_command_argument_count(), 0);

        command.push_command_parse_string(c"display-message");
        command.push_command_parse_string(c"hello");
        command.set_command_parse_command_line(18);
        assert_eq!(command.command_parse_command_line(), 18);
        assert_eq!(command.command_parse_command_argument_count(), 2);
        assert_eq!(
            command.command_parse_command_argument_kind(0),
            Some(CMD_PARSE_STRING)
        );
        assert_eq!(
            command.command_parse_command_argument_string(1),
            Some(c"hello")
        );

        command.clear_command_parse_arguments();
        assert_eq!(command.command_parse_command_argument_count(), 0);
    }
}

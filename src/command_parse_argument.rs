//! Portable state for one command-parser argument.

use core::ffi::{CStr, c_uint};

/// The discriminant and payload presence of one parser argument.
pub trait CommandParseArgument {
    /// Builds an argument with the supplied kind and optional string payload.
    fn from_command_parse_argument(kind: c_uint, string: Option<&CStr>) -> Self
    where
        Self: Sized;

    /// Returns the raw parser argument kind.
    fn command_parse_argument_kind(&self) -> c_uint;

    /// Replaces the raw parser argument kind.
    fn set_command_parse_argument_kind(&mut self, kind: c_uint);

    /// Returns the string payload, when present.
    fn command_parse_argument_string(&self) -> Option<&CStr>;

    /// Replaces or removes the string payload.
    fn set_command_parse_argument_string(&mut self, string: Option<&CStr>);

    /// Whether this argument owns an unparsed nested command tree.
    fn command_parse_argument_has_commands(&self) -> bool;

    /// Whether this argument owns an already parsed command list.
    fn command_parse_argument_has_command_list(&self) -> bool;
}

impl CommandParseArgument for crate::cmd::cmd_parse_argument {
    fn from_command_parse_argument(kind: c_uint, string: Option<&CStr>) -> Self {
        Self {
            type_0: kind,
            string: string.map(CStr::to_owned),
            commands: None,
            cmdlist: None,
        }
    }

    fn command_parse_argument_kind(&self) -> c_uint {
        self.type_0
    }

    fn set_command_parse_argument_kind(&mut self, kind: c_uint) {
        self.type_0 = kind;
    }

    fn command_parse_argument_string(&self) -> Option<&CStr> {
        self.string.as_deref()
    }

    fn set_command_parse_argument_string(&mut self, string: Option<&CStr>) {
        self.string = string.map(CStr::to_owned);
    }

    fn command_parse_argument_has_commands(&self) -> bool {
        self.commands.is_some()
    }

    fn command_parse_argument_has_command_list(&self) -> bool {
        self.cmdlist.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{CMD_PARSE_COMMANDS, CMD_PARSE_STRING, cmd_parse_argument};

    #[test]
    fn argument_kind_and_string_round_trip() {
        let mut argument =
            cmd_parse_argument::from_command_parse_argument(CMD_PARSE_STRING, Some(c"display"));
        assert_eq!(argument.command_parse_argument_kind(), CMD_PARSE_STRING);
        assert_eq!(argument.command_parse_argument_string(), Some(c"display"));
        assert!(!argument.command_parse_argument_has_commands());
        assert!(!argument.command_parse_argument_has_command_list());

        argument.set_command_parse_argument_kind(CMD_PARSE_COMMANDS);
        argument.set_command_parse_argument_string(Some(c"nested"));
        assert_eq!(argument.command_parse_argument_kind(), CMD_PARSE_COMMANDS);
        assert_eq!(argument.command_parse_argument_string(), Some(c"nested"));
        argument.set_command_parse_argument_string(None);
        assert_eq!(argument.command_parse_argument_string(), None);
    }
}

//! Source location retained by a command.

use crate::types::u_int;
use core::ffi::CStr;
use std::ffi::CString;

/// A command's optional source file and its line number.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CommandSource<'a> {
    pub file: Option<&'a CStr>,
    pub line: u_int,
}

/// Storage for where a command was parsed from.
pub trait CommandSourceState {
    /// Returns the current source location.
    fn command_source(&self) -> CommandSource<'_>;

    /// Copies a replacement source location.
    fn set_command_source(&mut self, source: CommandSource<'_>);
}

/// The command-source storage used by hmux.
#[derive(Default)]
pub struct RustCommandSourceState {
    file: Option<CString>,
    line: u_int,
}

impl CommandSourceState for RustCommandSourceState {
    fn command_source(&self) -> CommandSource<'_> {
        CommandSource {
            file: self.file.as_deref(),
            line: self.line,
        }
    }

    fn set_command_source(&mut self, source: CommandSource<'_>) {
        self.file = source.file.map(CStr::to_owned);
        self.line = source.line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_can_be_replaced_and_cleared() {
        let mut state = RustCommandSourceState::default();
        assert_eq!(
            state.command_source(),
            CommandSource {
                file: None,
                line: 0
            }
        );
        state.set_command_source(CommandSource {
            file: Some(c"tmux.conf"),
            line: 23,
        });
        assert_eq!(state.command_source().file, Some(c"tmux.conf"));
        assert_eq!(state.command_source().line, 23);
        state.set_command_source(CommandSource {
            file: None,
            line: 7,
        });
        assert_eq!(state.command_source().file, None);
        assert_eq!(state.command_source().line, 7);
    }
}

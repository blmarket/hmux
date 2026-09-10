//! Command metadata retained by a pane.

use std::ffi::CString;

/// The command, shell, and starting directory retained by a pane.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaneCommand {
    pub argv: Vec<CString>,
    pub shell: Option<CString>,
    pub cwd: Option<CString>,
}

/// Storage for a pane's retained command metadata.
pub trait PaneCommandState {
    /// Returns an owned snapshot of the command metadata.
    fn pane_command(&self) -> PaneCommand;

    /// Copies a complete replacement command.
    fn set_pane_command(&mut self, command: &PaneCommand);

    /// Clears all retained command metadata.
    fn clear_pane_command(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_can_be_replaced_and_cleared() {
        let mut state = crate::tests::test_fixtures::PaneAllocation::default();
        let command = PaneCommand {
            argv: vec![c"sh".to_owned(), c"-c".to_owned(), c"true".to_owned()],
            shell: Some(c"/bin/sh".to_owned()),
            cwd: Some(c"/tmp".to_owned()),
        };
        state.set_pane_command(&command);
        assert_eq!(state.pane_command(), command);
        state.clear_pane_command();
        assert_eq!(state.pane_command(), PaneCommand::default());
    }
}

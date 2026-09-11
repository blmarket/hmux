//! Command metadata retained by a pane.

use std::ffi::CString;

/// The command, shell, and starting directory retained by a pane.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaneCommand {
    pub argv: Vec<CString>,
    pub shell: Option<CString>,
    pub cwd: Option<CString>,
}

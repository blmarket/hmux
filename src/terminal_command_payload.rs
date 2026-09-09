//! Borrowed access to byte payloads carried by terminal commands.

use core::ffi::CStr;

/// A byte range carried by a terminal command.
///
/// ```compile_fail
/// use tmux_c2rs::{TerminalCommandData, types::tty_ctx_data};
/// fn escaped() -> tty_ctx_data<'static> {
///     let bytes = vec![1, 2, 3];
///     tty_ctx_data::from_terminal_command_data(&bytes)
/// }
/// ```
pub trait TerminalCommandData<'a> {
    /// Builds a terminal command borrowing its byte range.
    fn from_terminal_command_data(data: &'a [u8]) -> Self
    where
        Self: Sized;

    /// Borrows the complete byte range.
    fn terminal_command_data(&self) -> &[u8];

    /// Returns the number of bytes in the range.
    fn terminal_command_data_size(&self) -> usize;
}

/// A clipboard name and byte range carried by a terminal selection command.
///
/// ```compile_fail
/// use tmux_c2rs::{TerminalCommandSelection, types::tty_ctx_sel};
/// fn escaped() -> tty_ctx_sel<'static> {
///     let clip = c"clipboard".to_owned();
///     tty_ctx_sel::from_terminal_command_selection(&clip, b"abc")
/// }
/// ```
pub trait TerminalCommandSelection<'a> {
    /// Builds a terminal selection borrowing its clipboard name and bytes.
    fn from_terminal_command_selection(clip: &'a CStr, data: &'a [u8]) -> Self
    where
        Self: Sized;

    /// Borrows the clipboard name.
    fn terminal_command_clipboard(&self) -> &CStr;

    /// Borrows the complete selection byte range.
    fn terminal_command_selection_data(&self) -> &[u8];

    /// Returns the number of selection bytes.
    fn terminal_command_selection_size(&self) -> usize;
}

impl<'a> TerminalCommandData<'a> for crate::types::tty_ctx_data<'a> {
    fn from_terminal_command_data(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn terminal_command_data(&self) -> &[u8] {
        self.data
    }

    fn terminal_command_data_size(&self) -> usize {
        self.data.len()
    }
}

impl<'a> TerminalCommandSelection<'a> for crate::types::tty_ctx_sel<'a> {
    fn from_terminal_command_selection(clip: &'a CStr, data: &'a [u8]) -> Self {
        Self { clip, data }
    }

    fn terminal_command_clipboard(&self) -> &CStr {
        self.clip
    }

    fn terminal_command_selection_data(&self) -> &[u8] {
        self.data
    }

    fn terminal_command_selection_size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{tty_ctx_data, tty_ctx_sel};

    #[test]
    fn byte_range_round_trips() {
        let bytes = b"abc";
        let payload = tty_ctx_data::from_terminal_command_data(bytes);
        assert_eq!(payload.terminal_command_data(), bytes);
        assert_eq!(payload.terminal_command_data_size(), bytes.len());
    }

    #[test]
    fn selection_round_trips() {
        let clip = c"clipboard";
        let bytes = b"abc";
        let payload = tty_ctx_sel::from_terminal_command_selection(clip, bytes);
        assert_eq!(payload.terminal_command_clipboard(), clip);
        assert_eq!(payload.terminal_command_selection_data(), bytes);
        assert_eq!(payload.terminal_command_selection_size(), bytes.len());
    }
}

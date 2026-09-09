//! Stable access to the dimensions exchanged with terminal ioctls.

/// Terminal cell and pixel dimensions in the operating-system wire shape.
pub trait TerminalSize {
    /// Builds a terminal size from rows, columns, and total pixel dimensions.
    fn from_terminal_size(rows: u16, columns: u16, xpixel: u16, ypixel: u16) -> Self
    where
        Self: Sized;

    /// Returns the terminal height in rows.
    fn terminal_rows(&self) -> u16;

    /// Returns the terminal width in columns.
    fn terminal_columns(&self) -> u16;

    /// Returns the terminal's total pixel width.
    fn terminal_xpixel(&self) -> u16;

    /// Returns the terminal's total pixel height.
    fn terminal_ypixel(&self) -> u16;
}

impl TerminalSize for crate::types::winsize {
    fn from_terminal_size(rows: u16, columns: u16, xpixel: u16, ypixel: u16) -> Self {
        Self {
            ws_row: rows,
            ws_col: columns,
            ws_xpixel: xpixel,
            ws_ypixel: ypixel,
        }
    }

    fn terminal_rows(&self) -> u16 {
        self.ws_row
    }
    fn terminal_columns(&self) -> u16 {
        self.ws_col
    }
    fn terminal_xpixel(&self) -> u16 {
        self.ws_xpixel
    }
    fn terminal_ypixel(&self) -> u16 {
        self.ws_ypixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::winsize;

    #[test]
    fn cell_and_pixel_dimensions_round_trip() {
        let size = winsize::from_terminal_size(24, 80, 640, 384);
        assert_eq!(size.terminal_rows(), 24);
        assert_eq!(size.terminal_columns(), 80);
        assert_eq!(size.terminal_xpixel(), 640);
        assert_eq!(size.terminal_ypixel(), 384);
    }
}

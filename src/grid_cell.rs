//! Stable access to one terminal grid cell.

use crate::Utf8Data;

/// A terminal character and the style used to draw it.
pub trait GridCell: Utf8Data {
    /// Builds a grid cell from its complete portable state.
    fn from_grid_cell(
        bytes: [u8; 32],
        utf8_metadata: (u8, u8, u8),
        attributes: u16,
        flags: u8,
        colours: (i32, i32, i32),
        link: u32,
    ) -> Self;

    /// Returns the character attribute bits.
    fn grid_cell_attributes(&self) -> u16;

    /// Returns mutable access to the character attribute bits.
    fn grid_cell_attributes_mut(&mut self) -> &mut u16;

    /// Replaces the character attribute bits.
    fn set_grid_cell_attributes(&mut self, attributes: u16);

    /// Adds character attribute bits.
    fn add_grid_cell_attributes(&mut self, attributes: u16) {
        self.set_grid_cell_attributes(self.grid_cell_attributes() | attributes);
    }

    /// Removes character attribute bits.
    fn remove_grid_cell_attributes(&mut self, attributes: u16) {
        self.set_grid_cell_attributes(self.grid_cell_attributes() & !attributes);
    }

    /// Toggles character attribute bits.
    fn toggle_grid_cell_attributes(&mut self, attributes: u16) {
        self.set_grid_cell_attributes(self.grid_cell_attributes() ^ attributes);
    }

    /// Returns the cell flags.
    fn grid_cell_flags(&self) -> u8;

    /// Returns mutable access to the cell flags.
    fn grid_cell_flags_mut(&mut self) -> &mut u8;

    /// Replaces the cell flags.
    fn set_grid_cell_flags(&mut self, flags: u8);

    /// Adds cell flags.
    fn add_grid_cell_flags(&mut self, flags: u8) {
        self.set_grid_cell_flags(self.grid_cell_flags() | flags);
    }

    /// Removes cell flags.
    fn remove_grid_cell_flags(&mut self, flags: u8) {
        self.set_grid_cell_flags(self.grid_cell_flags() & !flags);
    }

    /// Toggles cell flags.
    fn toggle_grid_cell_flags(&mut self, flags: u8) {
        self.set_grid_cell_flags(self.grid_cell_flags() ^ flags);
    }

    /// Returns the foreground colour.
    fn grid_cell_foreground(&self) -> i32;

    /// Returns mutable access to the foreground colour.
    fn grid_cell_foreground_mut(&mut self) -> &mut i32;

    /// Replaces the foreground colour.
    fn set_grid_cell_foreground(&mut self, foreground: i32);

    /// Returns the background colour.
    fn grid_cell_background(&self) -> i32;

    /// Returns mutable access to the background colour.
    fn grid_cell_background_mut(&mut self) -> &mut i32;

    /// Replaces the background colour.
    fn set_grid_cell_background(&mut self, background: i32);

    /// Returns the underscore colour.
    fn grid_cell_underscore(&self) -> i32;

    /// Returns mutable access to the underscore colour.
    fn grid_cell_underscore_mut(&mut self) -> &mut i32;

    /// Replaces the underscore colour.
    fn set_grid_cell_underscore(&mut self, underscore: i32);

    /// Returns all three colours as foreground, background, and underscore.
    fn grid_cell_colours(&self) -> (i32, i32, i32) {
        (
            self.grid_cell_foreground(),
            self.grid_cell_background(),
            self.grid_cell_underscore(),
        )
    }

    /// Replaces foreground, background, and underscore together.
    fn set_grid_cell_colours(&mut self, foreground: i32, background: i32, underscore: i32) {
        self.set_grid_cell_foreground(foreground);
        self.set_grid_cell_background(background);
        self.set_grid_cell_underscore(underscore);
    }

    /// Returns the hyperlink identifier.
    fn grid_cell_link(&self) -> u32;

    /// Returns mutable access to the hyperlink identifier.
    fn grid_cell_link_mut(&mut self) -> &mut u32;

    /// Replaces the hyperlink identifier.
    fn set_grid_cell_link(&mut self, link: u32);
}

impl Utf8Data for crate::types::grid_cell {
    fn from_utf8_data(bytes: [u8; 32], have: u8, size: u8, width: u8) -> Self {
        Self {
            data: crate::text::utf8_data::from_utf8_data(bytes, have, size, width),
            ..Self::default()
        }
    }

    fn utf8_bytes(&self) -> &[u8; 32] {
        self.data.utf8_bytes()
    }

    fn utf8_bytes_mut(&mut self) -> &mut [u8; 32] {
        self.data.utf8_bytes_mut()
    }

    fn utf8_have(&self) -> u8 {
        self.data.utf8_have()
    }

    fn set_utf8_have(&mut self, have: u8) {
        self.data.set_utf8_have(have);
    }

    fn utf8_size(&self) -> u8 {
        self.data.utf8_size()
    }

    fn set_utf8_size(&mut self, size: u8) {
        self.data.set_utf8_size(size);
    }

    fn utf8_width(&self) -> u8 {
        self.data.utf8_width()
    }

    fn set_utf8_width(&mut self, width: u8) {
        self.data.set_utf8_width(width);
    }
}

impl GridCell for crate::types::grid_cell {
    fn from_grid_cell(
        bytes: [u8; 32],
        (have, size, width): (u8, u8, u8),
        attributes: u16,
        flags: u8,
        (foreground, background, underscore): (i32, i32, i32),
        link: u32,
    ) -> Self {
        Self {
            data: crate::text::utf8_data::from_utf8_data(bytes, have, size, width),
            attr: attributes,
            flags,
            fg: foreground,
            bg: background,
            us: underscore,
            link,
        }
    }

    fn grid_cell_attributes(&self) -> u16 {
        self.attr
    }

    fn grid_cell_attributes_mut(&mut self) -> &mut u16 {
        &mut self.attr
    }

    fn set_grid_cell_attributes(&mut self, attributes: u16) {
        self.attr = attributes;
    }

    fn grid_cell_flags(&self) -> u8 {
        self.flags
    }

    fn grid_cell_flags_mut(&mut self) -> &mut u8 {
        &mut self.flags
    }

    fn set_grid_cell_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    fn grid_cell_foreground(&self) -> i32 {
        self.fg
    }

    fn grid_cell_foreground_mut(&mut self) -> &mut i32 {
        &mut self.fg
    }

    fn set_grid_cell_foreground(&mut self, foreground: i32) {
        self.fg = foreground;
    }

    fn grid_cell_background(&self) -> i32 {
        self.bg
    }

    fn grid_cell_background_mut(&mut self) -> &mut i32 {
        &mut self.bg
    }

    fn set_grid_cell_background(&mut self, background: i32) {
        self.bg = background;
    }

    fn grid_cell_underscore(&self) -> i32 {
        self.us
    }

    fn grid_cell_underscore_mut(&mut self) -> &mut i32 {
        &mut self.us
    }

    fn set_grid_cell_underscore(&mut self, underscore: i32) {
        self.us = underscore;
    }

    fn grid_cell_link(&self) -> u32 {
        self.link
    }

    fn grid_cell_link_mut(&mut self) -> &mut u32 {
        &mut self.link
    }

    fn set_grid_cell_link(&mut self, link: u32) {
        self.link = link;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::grid_cell;

    #[test]
    fn complete_cell_round_trips_and_updates() {
        let mut bytes = [0; 32];
        bytes[..3].copy_from_slice("界".as_bytes());
        let mut cell = grid_cell::from_grid_cell(bytes, (3, 3, 2), 0x1234, 5, (6, 7, 8), 9);
        assert_eq!(&cell.utf8_bytes()[..3], "界".as_bytes());
        assert_eq!(
            (cell.utf8_have(), cell.utf8_size(), cell.utf8_width()),
            (3, 3, 2)
        );
        assert_eq!(cell.grid_cell_attributes(), 0x1234);
        assert_eq!(cell.grid_cell_flags(), 5);
        assert_eq!(cell.grid_cell_colours(), (6, 7, 8));
        assert_eq!(cell.grid_cell_link(), 9);

        cell.utf8_bytes_mut()[0] = b'x';
        cell.set_utf8_metadata(1, 1, 1);
        cell.set_grid_cell_attributes(10);
        cell.set_grid_cell_flags(11);
        cell.set_grid_cell_colours(12, 13, 14);
        cell.set_grid_cell_link(15);
        assert_eq!(cell.utf8_bytes()[0], b'x');
        assert_eq!(cell.grid_cell_attributes(), 10);
        assert_eq!(cell.grid_cell_flags(), 11);
        assert_eq!(cell.grid_cell_colours(), (12, 13, 14));
        assert_eq!(cell.grid_cell_link(), 15);
    }
}

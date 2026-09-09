//! Stable access to the packed extended-cell records stored by a grid line.

/// A complete extended grid cell payload.
pub trait GridExtendedEntry {
    /// Builds an extended cell entry.
    fn from_grid_extended_entry(
        data: u32,
        attributes: u16,
        flags: u8,
        foreground: i32,
        background: i32,
        underscore: i32,
        link: u32,
    ) -> Self
    where
        Self: Sized;

    /// Returns the encoded character.
    fn grid_extended_data(&self) -> u32;

    /// Replaces the encoded character.
    fn set_grid_extended_data(&mut self, data: u32);

    /// Returns the character attributes.
    fn grid_extended_attributes(&self) -> u16;

    /// Replaces the character attributes.
    fn set_grid_extended_attributes(&mut self, attributes: u16);

    /// Returns the cell flags.
    fn grid_extended_flags(&self) -> u8;

    /// Replaces the cell flags.
    fn set_grid_extended_flags(&mut self, flags: u8);

    /// Returns the foreground colour.
    fn grid_extended_foreground(&self) -> i32;

    /// Replaces the foreground colour.
    fn set_grid_extended_foreground(&mut self, foreground: i32);

    /// Returns the background colour.
    fn grid_extended_background(&self) -> i32;

    /// Replaces the background colour.
    fn set_grid_extended_background(&mut self, background: i32);

    /// Returns the underscore colour.
    fn grid_extended_underscore(&self) -> i32;

    /// Replaces the underscore colour.
    fn set_grid_extended_underscore(&mut self, underscore: i32);

    /// Returns the hyperlink identifier.
    fn grid_extended_link(&self) -> u32;

    /// Replaces the hyperlink identifier.
    fn set_grid_extended_link(&mut self, link: u32);
}

impl GridExtendedEntry for crate::types::grid_extd_entry {
    fn from_grid_extended_entry(
        data: u32,
        attributes: u16,
        flags: u8,
        foreground: i32,
        background: i32,
        underscore: i32,
        link: u32,
    ) -> Self {
        Self {
            data,
            attr: attributes,
            flags,
            fg: foreground,
            bg: background,
            us: underscore,
            link,
        }
    }

    fn grid_extended_data(&self) -> u32 {
        self.data
    }

    fn set_grid_extended_data(&mut self, data: u32) {
        self.data = data;
    }

    fn grid_extended_attributes(&self) -> u16 {
        self.attr
    }

    fn set_grid_extended_attributes(&mut self, attributes: u16) {
        self.attr = attributes;
    }

    fn grid_extended_flags(&self) -> u8 {
        self.flags
    }

    fn set_grid_extended_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    fn grid_extended_foreground(&self) -> i32 {
        self.fg
    }

    fn set_grid_extended_foreground(&mut self, foreground: i32) {
        self.fg = foreground;
    }

    fn grid_extended_background(&self) -> i32 {
        self.bg
    }

    fn set_grid_extended_background(&mut self, background: i32) {
        self.bg = background;
    }

    fn grid_extended_underscore(&self) -> i32 {
        self.us
    }

    fn set_grid_extended_underscore(&mut self, underscore: i32) {
        self.us = underscore;
    }

    fn grid_extended_link(&self) -> u32 {
        self.link
    }

    fn set_grid_extended_link(&mut self, link: u32) {
        self.link = link;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::grid_extd_entry;

    #[test]
    fn complete_payload_round_trips() {
        let mut entry = grid_extd_entry::from_grid_extended_entry(0x1f642, 0x1234, 5, 6, 7, 8, 9);
        assert_eq!(entry.grid_extended_data(), 0x1f642);
        assert_eq!(entry.grid_extended_attributes(), 0x1234);
        assert_eq!(entry.grid_extended_flags(), 5);
        assert_eq!(entry.grid_extended_foreground(), 6);
        assert_eq!(entry.grid_extended_background(), 7);
        assert_eq!(entry.grid_extended_underscore(), 8);
        assert_eq!(entry.grid_extended_link(), 9);
        entry.set_grid_extended_data(10);
        entry.set_grid_extended_attributes(11);
        entry.set_grid_extended_flags(12);
        entry.set_grid_extended_foreground(13);
        entry.set_grid_extended_background(14);
        entry.set_grid_extended_underscore(15);
        entry.set_grid_extended_link(16);
        assert_eq!(entry.grid_extended_data(), 10);
        assert_eq!(entry.grid_extended_attributes(), 11);
        assert_eq!(entry.grid_extended_flags(), 12);
        assert_eq!(entry.grid_extended_foreground(), 13);
        assert_eq!(entry.grid_extended_background(), 14);
        assert_eq!(entry.grid_extended_underscore(), 15);
        assert_eq!(entry.grid_extended_link(), 16);
    }
}

//! Stable access to one parsed style.

use crate::GridCell;
use core::ffi::c_char;

/// A styled terminal cell and its layout and range directives.
pub trait Style: GridCell {
    /// Builds a style from its complete portable state.
    #[allow(clippy::too_many_arguments)]
    fn from_style(
        bytes: [u8; 32],
        utf8_metadata: (u8, u8, u8),
        attributes: u16,
        flags: u8,
        colours: (i32, i32, i32),
        link: u32,
        ignore: i32,
        fill: i32,
        alignment: u32,
        list: u32,
        range_type: u32,
        range_argument: u32,
        range_string: [c_char; 16],
        width: i32,
        width_percentage: i32,
        padding: i32,
        default_type: u32,
    ) -> Self;

    /// Returns the ignore marker.
    fn style_ignore(&self) -> i32;

    /// Returns mutable access to the ignore marker.
    fn style_ignore_mut(&mut self) -> &mut i32;

    /// Replaces the ignore marker.
    fn set_style_ignore(&mut self, ignore: i32) {
        *self.style_ignore_mut() = ignore;
    }

    /// Returns the fill colour.
    fn style_fill(&self) -> i32;

    /// Returns mutable access to the fill colour.
    fn style_fill_mut(&mut self) -> &mut i32;

    /// Replaces the fill colour.
    fn set_style_fill(&mut self, fill: i32) {
        *self.style_fill_mut() = fill;
    }

    /// Returns the alignment mode.
    fn style_alignment(&self) -> u32;

    /// Returns mutable access to the alignment mode.
    fn style_alignment_mut(&mut self) -> &mut u32;

    /// Replaces the alignment mode.
    fn set_style_alignment(&mut self, alignment: u32) {
        *self.style_alignment_mut() = alignment;
    }

    /// Returns the list mode.
    fn style_list(&self) -> u32;

    /// Returns mutable access to the list mode.
    fn style_list_mut(&mut self) -> &mut u32;

    /// Replaces the list mode.
    fn set_style_list(&mut self, list: u32) {
        *self.style_list_mut() = list;
    }

    /// Returns the range kind.
    fn style_range_type(&self) -> u32;

    /// Returns mutable access to the range kind.
    fn style_range_type_mut(&mut self) -> &mut u32;

    /// Replaces the range kind.
    fn set_style_range_type(&mut self, range_type: u32) {
        *self.style_range_type_mut() = range_type;
    }

    /// Returns the numeric range argument.
    fn style_range_argument(&self) -> u32;

    /// Returns mutable access to the numeric range argument.
    fn style_range_argument_mut(&mut self) -> &mut u32;

    /// Replaces the numeric range argument.
    fn set_style_range_argument(&mut self, range_argument: u32) {
        *self.style_range_argument_mut() = range_argument;
    }

    /// Returns the textual range argument storage.
    fn style_range_string(&self) -> &[c_char; 16];

    /// Returns mutable access to the textual range argument storage.
    fn style_range_string_mut(&mut self) -> &mut [c_char; 16];

    /// Returns the requested width.
    fn style_width(&self) -> i32;

    /// Returns mutable access to the requested width.
    fn style_width_mut(&mut self) -> &mut i32;

    /// Replaces the requested width.
    fn set_style_width(&mut self, width: i32) {
        *self.style_width_mut() = width;
    }

    /// Returns the requested percentage width.
    fn style_width_percentage(&self) -> i32;

    /// Returns mutable access to the requested percentage width.
    fn style_width_percentage_mut(&mut self) -> &mut i32;

    /// Replaces the requested percentage width.
    fn set_style_width_percentage(&mut self, width_percentage: i32) {
        *self.style_width_percentage_mut() = width_percentage;
    }

    /// Returns the padding amount.
    fn style_padding(&self) -> i32;

    /// Returns mutable access to the padding amount.
    fn style_padding_mut(&mut self) -> &mut i32;

    /// Replaces the padding amount.
    fn set_style_padding(&mut self, padding: i32) {
        *self.style_padding_mut() = padding;
    }

    /// Returns the default-style operation.
    fn style_default_type(&self) -> u32;

    /// Returns mutable access to the default-style operation.
    fn style_default_type_mut(&mut self) -> &mut u32;

    /// Replaces the default-style operation.
    fn set_style_default_type(&mut self, default_type: u32) {
        *self.style_default_type_mut() = default_type;
    }
}

impl crate::Utf8Data for crate::types::style {
    fn from_utf8_data(bytes: [u8; 32], have: u8, size: u8, width: u8) -> Self {
        Self {
            gc: crate::types::grid_cell::from_utf8_data(bytes, have, size, width),
            ..Self::default()
        }
    }

    fn utf8_bytes(&self) -> &[u8; 32] {
        self.gc.utf8_bytes()
    }

    fn utf8_bytes_mut(&mut self) -> &mut [u8; 32] {
        self.gc.utf8_bytes_mut()
    }

    fn utf8_have(&self) -> u8 {
        self.gc.utf8_have()
    }

    fn set_utf8_have(&mut self, have: u8) {
        self.gc.set_utf8_have(have);
    }

    fn utf8_size(&self) -> u8 {
        self.gc.utf8_size()
    }

    fn set_utf8_size(&mut self, size: u8) {
        self.gc.set_utf8_size(size);
    }

    fn utf8_width(&self) -> u8 {
        self.gc.utf8_width()
    }

    fn set_utf8_width(&mut self, width: u8) {
        self.gc.set_utf8_width(width);
    }
}

impl GridCell for crate::types::style {
    fn from_grid_cell(
        bytes: [u8; 32],
        utf8_metadata: (u8, u8, u8),
        attributes: u16,
        flags: u8,
        colours: (i32, i32, i32),
        link: u32,
    ) -> Self {
        Self {
            gc: crate::types::grid_cell::from_grid_cell(
                bytes,
                utf8_metadata,
                attributes,
                flags,
                colours,
                link,
            ),
            ..Self::default()
        }
    }

    fn grid_cell_attributes(&self) -> u16 {
        self.gc.grid_cell_attributes()
    }

    fn grid_cell_attributes_mut(&mut self) -> &mut u16 {
        self.gc.grid_cell_attributes_mut()
    }

    fn set_grid_cell_attributes(&mut self, attributes: u16) {
        self.gc.set_grid_cell_attributes(attributes);
    }

    fn grid_cell_flags(&self) -> u8 {
        self.gc.grid_cell_flags()
    }

    fn grid_cell_flags_mut(&mut self) -> &mut u8 {
        self.gc.grid_cell_flags_mut()
    }

    fn set_grid_cell_flags(&mut self, flags: u8) {
        self.gc.set_grid_cell_flags(flags);
    }

    fn grid_cell_foreground(&self) -> i32 {
        self.gc.grid_cell_foreground()
    }

    fn grid_cell_foreground_mut(&mut self) -> &mut i32 {
        self.gc.grid_cell_foreground_mut()
    }

    fn set_grid_cell_foreground(&mut self, foreground: i32) {
        self.gc.set_grid_cell_foreground(foreground);
    }

    fn grid_cell_background(&self) -> i32 {
        self.gc.grid_cell_background()
    }

    fn grid_cell_background_mut(&mut self) -> &mut i32 {
        self.gc.grid_cell_background_mut()
    }

    fn set_grid_cell_background(&mut self, background: i32) {
        self.gc.set_grid_cell_background(background);
    }

    fn grid_cell_underscore(&self) -> i32 {
        self.gc.grid_cell_underscore()
    }

    fn grid_cell_underscore_mut(&mut self) -> &mut i32 {
        self.gc.grid_cell_underscore_mut()
    }

    fn set_grid_cell_underscore(&mut self, underscore: i32) {
        self.gc.set_grid_cell_underscore(underscore);
    }

    fn grid_cell_link(&self) -> u32 {
        self.gc.grid_cell_link()
    }

    fn grid_cell_link_mut(&mut self) -> &mut u32 {
        self.gc.grid_cell_link_mut()
    }

    fn set_grid_cell_link(&mut self, link: u32) {
        self.gc.set_grid_cell_link(link);
    }
}

impl Style for crate::types::style {
    fn from_style(
        bytes: [u8; 32],
        utf8_metadata: (u8, u8, u8),
        attributes: u16,
        flags: u8,
        colours: (i32, i32, i32),
        link: u32,
        ignore: i32,
        fill: i32,
        alignment: u32,
        list: u32,
        range_type: u32,
        range_argument: u32,
        range_string: [c_char; 16],
        width: i32,
        width_percentage: i32,
        padding: i32,
        default_type: u32,
    ) -> Self {
        Self {
            gc: crate::types::grid_cell::from_grid_cell(
                bytes,
                utf8_metadata,
                attributes,
                flags,
                colours,
                link,
            ),
            ignore,
            fill,
            align: alignment,
            list,
            range_type,
            range_argument,
            range_string,
            width,
            width_percentage,
            pad: padding,
            default_type,
        }
    }

    fn style_ignore(&self) -> i32 {
        self.ignore
    }

    fn style_ignore_mut(&mut self) -> &mut i32 {
        &mut self.ignore
    }

    fn style_fill(&self) -> i32 {
        self.fill
    }

    fn style_fill_mut(&mut self) -> &mut i32 {
        &mut self.fill
    }

    fn style_alignment(&self) -> u32 {
        self.align
    }

    fn style_alignment_mut(&mut self) -> &mut u32 {
        &mut self.align
    }

    fn style_list(&self) -> u32 {
        self.list
    }

    fn style_list_mut(&mut self) -> &mut u32 {
        &mut self.list
    }

    fn style_range_type(&self) -> u32 {
        self.range_type
    }

    fn style_range_type_mut(&mut self) -> &mut u32 {
        &mut self.range_type
    }

    fn style_range_argument(&self) -> u32 {
        self.range_argument
    }

    fn style_range_argument_mut(&mut self) -> &mut u32 {
        &mut self.range_argument
    }

    fn style_range_string(&self) -> &[c_char; 16] {
        &self.range_string
    }

    fn style_range_string_mut(&mut self) -> &mut [c_char; 16] {
        &mut self.range_string
    }

    fn style_width(&self) -> i32 {
        self.width
    }

    fn style_width_mut(&mut self) -> &mut i32 {
        &mut self.width
    }

    fn style_width_percentage(&self) -> i32 {
        self.width_percentage
    }

    fn style_width_percentage_mut(&mut self) -> &mut i32 {
        &mut self.width_percentage
    }

    fn style_padding(&self) -> i32 {
        self.pad
    }

    fn style_padding_mut(&mut self) -> &mut i32 {
        &mut self.pad
    }

    fn style_default_type(&self) -> u32 {
        self.default_type
    }

    fn style_default_type_mut(&mut self) -> &mut u32 {
        &mut self.default_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Utf8Data;
    use crate::types::style;

    #[test]
    fn complete_style_round_trips_and_updates() {
        let mut bytes = [0; 32];
        bytes[0] = b'x';
        let mut range_string = [0; 16];
        range_string[0] = b'r' as c_char;
        let mut value = style::from_style(
            bytes,
            (1, 1, 1),
            2,
            3,
            (4, 5, 6),
            7,
            8,
            9,
            10,
            11,
            12,
            13,
            range_string,
            14,
            15,
            16,
            17,
        );
        assert_eq!(value.utf8_bytes()[0], b'x');
        assert_eq!(value.grid_cell_colours(), (4, 5, 6));
        assert_eq!(value.style_ignore(), 8);
        assert_eq!(value.style_fill(), 9);
        assert_eq!(value.style_alignment(), 10);
        assert_eq!(value.style_list(), 11);
        assert_eq!(value.style_range_type(), 12);
        assert_eq!(value.style_range_argument(), 13);
        assert_eq!(value.style_range_string()[0], b'r' as c_char);
        assert_eq!(value.style_width(), 14);
        assert_eq!(value.style_width_percentage(), 15);
        assert_eq!(value.style_padding(), 16);
        assert_eq!(value.style_default_type(), 17);

        value.set_style_ignore(18);
        value.set_style_fill(19);
        value.set_style_alignment(20);
        value.set_style_list(21);
        value.set_style_range_type(22);
        value.set_style_range_argument(23);
        value.style_range_string_mut()[0] = b's' as c_char;
        value.set_style_width(24);
        value.set_style_width_percentage(25);
        value.set_style_padding(26);
        value.set_style_default_type(27);
        assert_eq!(value.style_ignore(), 18);
        assert_eq!(value.style_fill(), 19);
        assert_eq!(value.style_alignment(), 20);
        assert_eq!(value.style_list(), 21);
        assert_eq!(value.style_range_type(), 22);
        assert_eq!(value.style_range_argument(), 23);
        assert_eq!(value.style_range_string()[0], b's' as c_char);
        assert_eq!(value.style_width(), 24);
        assert_eq!(value.style_width_percentage(), 25);
        assert_eq!(value.style_padding(), 26);
        assert_eq!(value.style_default_type(), 27);
    }
}

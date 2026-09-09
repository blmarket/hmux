//! Stable access to a range produced while drawing formatted text.

use core::ffi::c_char;

/// A typed half-open span in a formatted status line.
pub trait StyleRange {
    /// Builds a style range.
    fn from_style_range(
        kind: u32,
        argument: u32,
        string: [c_char; 16],
        start: u32,
        end: u32,
    ) -> Self
    where
        Self: Sized;

    /// Returns the range kind.
    fn style_range_kind(&self) -> u32;

    /// Returns the numeric range argument.
    fn style_range_argument(&self) -> u32;

    /// Returns the fixed string argument.
    fn style_range_string(&self) -> &[c_char; 16];

    /// Returns the included start position.
    fn style_range_start(&self) -> u32;

    /// Returns the excluded end position.
    fn style_range_end(&self) -> u32;
}

impl StyleRange for crate::types::style_range {
    fn from_style_range(
        kind: u32,
        argument: u32,
        string: [c_char; 16],
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            type_0: kind,
            argument,
            string,
            start,
            end,
        }
    }

    fn style_range_kind(&self) -> u32 {
        self.type_0
    }

    fn style_range_argument(&self) -> u32 {
        self.argument
    }

    fn style_range_string(&self) -> &[c_char; 16] {
        &self.string
    }

    fn style_range_start(&self) -> u32 {
        self.start
    }

    fn style_range_end(&self) -> u32 {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::style_range;

    #[test]
    fn range_fields_round_trip() {
        let mut string = [0; 16];
        string[0] = b'x' as c_char;
        let range = style_range::from_style_range(3, 9, string, 4, 12);
        assert_eq!(range.style_range_kind(), 3);
        assert_eq!(range.style_range_argument(), 9);
        assert_eq!(range.style_range_string(), &string);
        assert_eq!(range.style_range_start(), 4);
        assert_eq!(range.style_range_end(), 12);
    }
}

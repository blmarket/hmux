//! Stable access to cached formatted lines and their interactive ranges.

use core::ffi::{CStr, c_char};
use std::ffi::CString;

/// The cached expansion and ranges for one formatted status line.
pub trait StyleLine {
    /// Builds an empty range list with an optional cached expansion.
    fn from_style_line(expanded: Option<CString>) -> Self
    where
        Self: Sized;

    /// Returns the cached expansion.
    fn style_line_expanded(&self) -> Option<&CStr>;

    /// Replaces the cached expansion and returns the previous value.
    fn replace_style_line_expanded(&mut self, expanded: Option<CString>) -> Option<CString>;

    /// Removes every interactive range.
    fn clear_style_line_ranges(&mut self);

    /// Returns the number of interactive ranges.
    fn style_line_range_count(&self) -> usize;

    /// Returns one range as kind, argument, string, start, and end.
    fn style_line_range_at(&self, index: usize) -> Option<(u32, u32, [c_char; 16], u32, u32)>;

    /// Appends one interactive range.
    fn push_style_line_range(
        &mut self,
        kind: u32,
        argument: u32,
        string: [c_char; 16],
        start: u32,
        end: u32,
    );
}

impl StyleLine for crate::types::style_line_entry {
    fn from_style_line(expanded: Option<CString>) -> Self {
        Self {
            expanded,
            ranges: Vec::new(),
        }
    }

    fn style_line_expanded(&self) -> Option<&CStr> {
        self.expanded.as_deref()
    }

    fn replace_style_line_expanded(&mut self, expanded: Option<CString>) -> Option<CString> {
        core::mem::replace(&mut self.expanded, expanded)
    }

    fn clear_style_line_ranges(&mut self) {
        self.ranges.clear();
    }

    fn style_line_range_count(&self) -> usize {
        self.ranges.len()
    }

    fn style_line_range_at(&self, index: usize) -> Option<(u32, u32, [c_char; 16], u32, u32)> {
        let range = self.ranges.get(index)?;
        Some((
            range.type_0,
            range.argument,
            range.string,
            range.start,
            range.end,
        ))
    }

    fn push_style_line_range(
        &mut self,
        kind: u32,
        argument: u32,
        string: [c_char; 16],
        start: u32,
        end: u32,
    ) {
        self.ranges.push(crate::types::style_range {
            type_0: kind,
            argument,
            string,
            start,
            end,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::style_line_entry;

    #[test]
    fn expansion_and_ranges_update() {
        let mut line = style_line_entry::from_style_line(Some(c"old".to_owned()));
        assert_eq!(line.style_line_expanded(), Some(c"old"));
        assert_eq!(
            line.replace_style_line_expanded(Some(c"new".to_owned())),
            Some(c"old".to_owned())
        );
        line.push_style_line_range(3, 9, [0; 16], 4, 12);
        assert_eq!(line.style_line_range_at(0), Some((3, 9, [0; 16], 4, 12)));
        line.clear_style_line_ranges();
        assert_eq!(line.style_line_range_count(), 0);
    }
}

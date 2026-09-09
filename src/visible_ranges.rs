//! Stable access to visible spans and their reusable collection.

use crate::types::u_int;

/// One visible span, expressed as a start column and length.
pub trait VisibleRange {
    /// Builds a visible span.
    fn from_visible_range(start: u_int, length: u_int) -> Self
    where
        Self: Sized;

    /// Returns the first visible column.
    fn visible_range_start(&self) -> u_int;

    /// Returns the number of visible columns.
    fn visible_range_length(&self) -> u_int;

    /// Replaces the span's start and length.
    fn set_visible_range(&mut self, start: u_int, length: u_int);
}

/// Reusable storage for the visible spans in one row.
pub trait VisibleRanges {
    /// Builds a collection whose supplied spans are all active.
    fn from_visible_ranges(ranges: &[(u_int, u_int)]) -> Self
    where
        Self: Sized;

    /// Returns the allocated number of span slots.
    fn visible_range_capacity(&self) -> usize;

    /// Ensures at least `capacity` span slots exist.
    fn ensure_visible_range_capacity(&mut self, capacity: usize);

    /// Returns the number of active spans.
    fn visible_range_count(&self) -> usize;

    /// Sets the number of active spans.
    ///
    /// # Panics
    ///
    /// Panics when `count` exceeds the allocated span slots.
    fn set_visible_range_count(&mut self, count: usize);

    /// Returns the start and length in the allocated slot at `index`.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the allocated span slots.
    fn visible_range_at(&self, index: usize) -> (u_int, u_int);

    /// Replaces the start and length in the allocated slot at `index`.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the allocated span slots.
    fn set_visible_range_at(&mut self, index: usize, start: u_int, length: u_int);

    /// Clears the active spans while retaining allocated storage.
    fn clear_visible_ranges(&mut self) {
        self.set_visible_range_count(0);
    }

    /// Releases all span storage and clears the active spans.
    fn release_visible_ranges(&mut self);
}

impl VisibleRange for crate::types::visible_range {
    fn from_visible_range(start: u_int, length: u_int) -> Self {
        Self {
            px: start,
            nx: length,
        }
    }

    fn visible_range_start(&self) -> u_int {
        self.px
    }

    fn visible_range_length(&self) -> u_int {
        self.nx
    }

    fn set_visible_range(&mut self, start: u_int, length: u_int) {
        self.px = start;
        self.nx = length;
    }
}

impl VisibleRanges for crate::types::visible_ranges {
    fn from_visible_ranges(ranges: &[(u_int, u_int)]) -> Self {
        Self {
            ranges: ranges
                .iter()
                .map(|&(start, length)| crate::types::visible_range {
                    px: start,
                    nx: length,
                })
                .collect(),
            used: ranges.len() as u_int,
        }
    }

    fn visible_range_capacity(&self) -> usize {
        self.ranges.len()
    }

    fn ensure_visible_range_capacity(&mut self, capacity: usize) {
        self.ranges
            .resize_with(capacity, || crate::types::visible_range { px: 0, nx: 0 });
    }

    fn visible_range_count(&self) -> usize {
        self.used as usize
    }

    fn set_visible_range_count(&mut self, count: usize) {
        assert!(count <= self.ranges.len());
        self.used = count as u_int;
    }

    fn visible_range_at(&self, index: usize) -> (u_int, u_int) {
        let range = &self.ranges[index];
        (range.px, range.nx)
    }

    fn set_visible_range_at(&mut self, index: usize, start: u_int, length: u_int) {
        let range = &mut self.ranges[index];
        range.px = start;
        range.nx = length;
    }

    fn release_visible_ranges(&mut self) {
        self.ranges = Vec::new();
        self.used = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{visible_range, visible_ranges};

    #[test]
    fn collection_reuses_slots_and_tracks_active_ranges() {
        let mut ranges = visible_ranges::default();
        ranges.ensure_visible_range_capacity(2);
        ranges.set_visible_range_at(0, 3, 5);
        ranges.set_visible_range_at(1, 12, 4);
        ranges.set_visible_range_count(2);

        assert_eq!(ranges.visible_range_capacity(), 2);
        assert_eq!(ranges.visible_range_count(), 2);
        assert_eq!(ranges.visible_range_at(0), (3, 5));
        assert_eq!(ranges.visible_range_at(1), (12, 4));
        ranges.clear_visible_ranges();
        assert_eq!(ranges.visible_range_count(), 0);

        let mut range = visible_range::from_visible_range(7, 9);
        assert_eq!(
            (range.visible_range_start(), range.visible_range_length()),
            (7, 9)
        );
        range.set_visible_range(1, 2);
        assert_eq!(
            (range.visible_range_start(), range.visible_range_length()),
            (1, 2)
        );
    }
}

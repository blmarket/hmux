//! Stable access to an owned line of compact and extended grid cells.

use crate::types::{grid_cell_entry, grid_extd_entry, time_t, u_int};
use core::ffi::c_int;

/// One owned grid line and all of its C record state.
pub trait GridLine {
    /// Builds a line by copying its compact and extended entries.
    fn from_grid_line(
        cells: &[grid_cell_entry],
        cell_used: u_int,
        extended: &[grid_extd_entry],
        flags: c_int,
        timestamp: time_t,
    ) -> Self
    where
        Self: Sized;

    /// Returns the compact cell allocation.
    fn grid_line_cells(&self) -> &[grid_cell_entry];

    /// Returns the compact cell allocation for mutation.
    fn grid_line_cells_mut(&mut self) -> &mut [grid_cell_entry];

    /// Returns how many compact cells have been written.
    fn grid_line_cell_used(&self) -> u_int;

    /// Replaces the number of compact cells which have been written.
    fn set_grid_line_cell_used(&mut self, cell_used: u_int);

    /// Returns the compact cell allocation size.
    fn grid_line_cell_capacity(&self) -> u_int {
        self.grid_line_cells().len() as u_int
    }

    /// Returns the extended cell allocation.
    fn grid_line_extended(&self) -> &[grid_extd_entry];

    /// Returns the extended cell allocation for mutation.
    fn grid_line_extended_mut(&mut self) -> &mut [grid_extd_entry];

    /// Returns the extended cell allocation size.
    fn grid_line_extended_count(&self) -> u_int {
        self.grid_line_extended().len() as u_int
    }

    /// Returns both cell allocations for mutation.
    fn grid_line_parts_mut(&mut self) -> (&mut [grid_cell_entry], &mut [grid_extd_entry]);

    /// Resizes the compact cell allocation, filling new entries with empty cells.
    fn resize_grid_line_cells(&mut self, size: u_int);

    /// Appends an empty extended entry and returns its index.
    fn push_grid_line_extended(&mut self) -> u_int;

    /// Replaces the extended cell allocation by copying `entries`.
    fn set_grid_line_extended(&mut self, entries: &[grid_extd_entry]);

    /// Returns the line flags.
    fn grid_line_flags(&self) -> c_int;

    /// Replaces the line flags.
    fn set_grid_line_flags(&mut self, flags: c_int);

    /// Adds line flags.
    fn add_grid_line_flags(&mut self, flags: c_int) {
        self.set_grid_line_flags(self.grid_line_flags() | flags);
    }

    /// Removes line flags.
    fn remove_grid_line_flags(&mut self, flags: c_int) {
        self.set_grid_line_flags(self.grid_line_flags() & !flags);
    }

    /// Returns the line timestamp.
    fn grid_line_time(&self) -> time_t;

    /// Replaces the line timestamp.
    fn set_grid_line_time(&mut self, timestamp: time_t);
}

impl GridLine for crate::grid::grid_line {
    fn from_grid_line(
        cells: &[crate::types::grid_cell_entry],
        cell_used: u_int,
        extended: &[crate::types::grid_extd_entry],
        flags: c_int,
        timestamp: crate::types::time_t,
    ) -> Self {
        let mut line = Self::new();
        line.resize_cells(cells.len() as u_int);
        line.celldata_mut().copy_from_slice(cells);
        line.cellused = cell_used;
        line.set_extended(extended);
        line.flags = flags;
        line.time = timestamp;
        line
    }

    fn grid_line_cells(&self) -> &[crate::types::grid_cell_entry] {
        self.celldata()
    }

    fn grid_line_cells_mut(&mut self) -> &mut [crate::types::grid_cell_entry] {
        self.celldata_mut()
    }

    fn grid_line_cell_used(&self) -> u_int {
        self.cellused
    }

    fn set_grid_line_cell_used(&mut self, cell_used: u_int) {
        self.cellused = cell_used;
    }

    fn grid_line_extended(&self) -> &[crate::types::grid_extd_entry] {
        self.extddata()
    }

    fn grid_line_extended_mut(&mut self) -> &mut [crate::types::grid_extd_entry] {
        self.extddata_mut()
    }

    fn grid_line_parts_mut(
        &mut self,
    ) -> (
        &mut [crate::types::grid_cell_entry],
        &mut [crate::types::grid_extd_entry],
    ) {
        self.parts_mut()
    }

    fn resize_grid_line_cells(&mut self, size: u_int) {
        self.resize_cells(size);
    }

    fn push_grid_line_extended(&mut self) -> u_int {
        self.push_extended()
    }

    fn set_grid_line_extended(&mut self, entries: &[crate::types::grid_extd_entry]) {
        self.set_extended(entries);
    }

    fn grid_line_flags(&self) -> c_int {
        self.flags
    }

    fn set_grid_line_flags(&mut self, flags: c_int) {
        self.flags = flags;
    }

    fn grid_line_time(&self) -> crate::types::time_t {
        self.time
    }

    fn set_grid_line_time(&mut self, timestamp: crate::types::time_t) {
        self.time = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::grid_line;
    use crate::{GridCellEntry, GridExtendedEntry};

    #[test]
    fn complete_line_state_and_allocations_round_trip() {
        let cells = [
            grid_cell_entry::from_compact_grid_cell_entry([1, 2, 3, 4], 5),
            grid_cell_entry::from_extended_grid_cell_entry(0, 6),
        ];
        let extended = [grid_extd_entry::from_grid_extended_entry(
            7, 8, 9, 10, 11, 12, 13,
        )];
        let mut line = grid_line::from_grid_line(&cells, 1, &extended, 14, 15);

        assert_eq!(line.grid_line_cell_capacity(), 2);
        assert_eq!(line.grid_line_cell_used(), 1);
        assert_eq!(line.grid_line_extended_count(), 1);
        assert_eq!(line.grid_line_flags(), 14);
        assert_eq!(line.grid_line_time(), 15);
        assert_eq!(
            line.grid_line_cells()[0].grid_cell_entry_data(),
            [1, 2, 3, 4]
        );
        assert_eq!(line.grid_line_extended()[0].grid_extended_data(), 7);

        line.set_grid_line_cell_used(2);
        line.add_grid_line_flags(0x10);
        line.remove_grid_line_flags(0x02);
        line.set_grid_line_time(16);
        line.resize_grid_line_cells(3);
        assert_eq!(line.push_grid_line_extended(), 1);
        let (cells, extended) = line.grid_line_parts_mut();
        cells[2].set_grid_cell_entry_flags(17);
        extended[1].set_grid_extended_data(18);

        assert_eq!(line.grid_line_cell_used(), 2);
        assert_eq!(line.grid_line_cell_capacity(), 3);
        assert_eq!(line.grid_line_cells()[2].grid_cell_entry_flags(), 17);
        assert_eq!(line.grid_line_extended_count(), 2);
        assert_eq!(line.grid_line_extended()[1].grid_extended_data(), 18);
        assert_eq!(line.grid_line_flags(), 0x1c);
        assert_eq!(line.grid_line_time(), 16);
    }
}

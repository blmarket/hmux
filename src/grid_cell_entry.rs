//! Stable access to the packed cell records stored by a grid line.

/// A compact cell payload or extended-cell offset, plus its flags.
pub trait GridCellEntry {
    /// Builds an entry containing compact attribute, foreground, background, and byte data.
    fn from_compact_grid_cell_entry(data: [u8; 4], flags: u8) -> Self
    where
        Self: Sized;

    /// Builds an entry pointing at an extended cell.
    fn from_extended_grid_cell_entry(offset: u32, flags: u8) -> Self
    where
        Self: Sized;

    /// Returns the four compact payload bytes.
    fn grid_cell_entry_data(&self) -> [u8; 4];

    /// Replaces the compact payload bytes.
    fn set_grid_cell_entry_data(&mut self, data: [u8; 4]);

    /// Returns the extended-cell offset encoded in the payload.
    fn grid_cell_entry_offset(&self) -> u32;

    /// Replaces the extended-cell offset encoded in the payload.
    fn set_grid_cell_entry_offset(&mut self, offset: u32);

    /// Returns the entry flags.
    fn grid_cell_entry_flags(&self) -> u8;

    /// Replaces the entry flags.
    fn set_grid_cell_entry_flags(&mut self, flags: u8);
}

impl GridCellEntry for crate::types::grid_cell_entry {
    fn from_compact_grid_cell_entry(data: [u8; 4], flags: u8) -> Self {
        Self {
            c2rust_unnamed: crate::types::grid_cell_entry_union {
                data: crate::types::grid_cell_entry_data {
                    attr: data[0],
                    fg: data[1],
                    bg: data[2],
                    data: data[3],
                },
            },
            flags,
        }
    }

    fn from_extended_grid_cell_entry(offset: u32, flags: u8) -> Self {
        Self {
            c2rust_unnamed: crate::types::grid_cell_entry_union { offset },
            flags,
        }
    }

    fn grid_cell_entry_data(&self) -> [u8; 4] {
        let data = unsafe { self.c2rust_unnamed.data };
        [data.attr, data.fg, data.bg, data.data]
    }

    fn set_grid_cell_entry_data(&mut self, data: [u8; 4]) {
        self.c2rust_unnamed.data = crate::types::grid_cell_entry_data {
            attr: data[0],
            fg: data[1],
            bg: data[2],
            data: data[3],
        };
    }

    fn grid_cell_entry_offset(&self) -> u32 {
        unsafe { self.c2rust_unnamed.offset }
    }

    fn set_grid_cell_entry_offset(&mut self, offset: u32) {
        self.c2rust_unnamed.offset = offset;
    }

    fn grid_cell_entry_flags(&self) -> u8 {
        self.flags
    }

    fn set_grid_cell_entry_flags(&mut self, flags: u8) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::grid_cell_entry;

    #[test]
    fn compact_and_extended_payloads_round_trip() {
        let mut entry = grid_cell_entry::from_compact_grid_cell_entry([1, 2, 3, 4], 5);
        assert_eq!(entry.grid_cell_entry_data(), [1, 2, 3, 4]);
        assert_eq!(entry.grid_cell_entry_flags(), 5);
        entry.set_grid_cell_entry_data([6, 7, 8, 9]);
        entry.set_grid_cell_entry_flags(10);
        assert_eq!(entry.grid_cell_entry_data(), [6, 7, 8, 9]);
        assert_eq!(entry.grid_cell_entry_flags(), 10);

        let mut extended = grid_cell_entry::from_extended_grid_cell_entry(17, 1);
        assert_eq!(extended.grid_cell_entry_offset(), 17);
        extended.set_grid_cell_entry_offset(23);
        assert_eq!(extended.grid_cell_entry_offset(), 23);
    }
}

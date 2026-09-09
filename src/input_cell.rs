//! Stable access to the cell assembled by the input parser.

use crate::types::grid_cell;
use core::ffi::c_int;

/// A terminal cell and the character-set state used to assemble it.
pub trait InputCell {
    /// Builds a parser cell from its terminal cell and character-set state.
    fn from_input_cell(cell: grid_cell, set: c_int, g0set: c_int, g1set: c_int) -> Self
    where
        Self: Sized;

    /// Returns the terminal cell being assembled.
    fn input_grid_cell(&self) -> &grid_cell;

    /// Returns the terminal cell being assembled for mutation.
    fn input_grid_cell_mut(&mut self) -> &mut grid_cell;

    /// Returns the selected character set and whether G0 and G1 use ACS.
    fn input_character_sets(&self) -> (c_int, c_int, c_int);

    /// Replaces the selected character set and the G0 and G1 ACS flags.
    fn set_input_character_sets(&mut self, set: c_int, g0set: c_int, g1set: c_int);

    /// Returns which character set is selected.
    fn input_character_set(&self) -> c_int {
        self.input_character_sets().0
    }

    /// Selects a character set.
    fn set_input_character_set(&mut self, set: c_int) {
        let (_, g0set, g1set) = self.input_character_sets();
        self.set_input_character_sets(set, g0set, g1set);
    }

    /// Returns whether G0 uses the alternate character set.
    fn input_g0_uses_acs(&self) -> c_int {
        self.input_character_sets().1
    }

    /// Sets whether G0 uses the alternate character set.
    fn set_input_g0_uses_acs(&mut self, g0set: c_int) {
        let (set, _, g1set) = self.input_character_sets();
        self.set_input_character_sets(set, g0set, g1set);
    }

    /// Returns whether G1 uses the alternate character set.
    fn input_g1_uses_acs(&self) -> c_int {
        self.input_character_sets().2
    }

    /// Sets whether G1 uses the alternate character set.
    fn set_input_g1_uses_acs(&mut self, g1set: c_int) {
        let (set, g0set, _) = self.input_character_sets();
        self.set_input_character_sets(set, g0set, g1set);
    }
}

impl InputCell for crate::types::input_cell {
    fn from_input_cell(cell: grid_cell, set: c_int, g0set: c_int, g1set: c_int) -> Self {
        Self {
            cell,
            set,
            g0set,
            g1set,
        }
    }
    fn input_grid_cell(&self) -> &grid_cell {
        &self.cell
    }
    fn input_grid_cell_mut(&mut self) -> &mut grid_cell {
        &mut self.cell
    }
    fn input_character_sets(&self) -> (c_int, c_int, c_int) {
        (self.set, self.g0set, self.g1set)
    }
    fn set_input_character_sets(&mut self, set: c_int, g0set: c_int, g1set: c_int) {
        self.set = set;
        self.g0set = g0set;
        self.g1set = g1set;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridCell;
    use crate::types::input_cell;

    #[test]
    fn cell_and_character_sets_round_trip() {
        let mut value = input_cell::from_input_cell(grid_cell::default(), 0, 1, 0);
        value.input_grid_cell_mut().set_grid_cell_foreground(7);
        value.set_input_character_sets(1, 0, 1);
        assert_eq!(value.input_grid_cell().grid_cell_foreground(), 7);
        assert_eq!(value.input_character_sets(), (1, 0, 1));
    }
}

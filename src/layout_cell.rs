//! Stable access to a cell in a window layout tree.

/// The portable state of one window layout cell.
pub trait LayoutCell {
    /// Builds a detached cell with no children.
    fn from_layout_cell(cell_type: u32, flags: i32, size: (u32, u32), offset: (i32, i32)) -> Self
    where
        Self: Sized;

    /// Returns the layout cell type.
    fn layout_cell_type(&self) -> u32;

    /// Sets the layout cell type.
    fn set_layout_cell_type(&mut self, cell_type: u32);

    /// Returns the layout cell flags.
    fn layout_cell_flags(&self) -> i32;

    /// Sets the layout cell flags.
    fn set_layout_cell_flags(&mut self, flags: i32);

    /// Adds flags to the layout cell.
    fn add_layout_cell_flags(&mut self, flags: i32) {
        self.set_layout_cell_flags(self.layout_cell_flags() | flags);
    }

    /// Removes flags from the layout cell.
    fn remove_layout_cell_flags(&mut self, flags: i32) {
        self.set_layout_cell_flags(self.layout_cell_flags() & !flags);
    }

    /// Toggles flags on the layout cell.
    fn toggle_layout_cell_flags(&mut self, flags: i32) {
        self.set_layout_cell_flags(self.layout_cell_flags() ^ flags);
    }

    /// Returns the cell width and height.
    fn layout_cell_size(&self) -> (u32, u32);

    /// Sets the cell width and height.
    fn set_layout_cell_size(&mut self, width: u32, height: u32);

    /// Returns the cell width.
    fn layout_cell_width(&self) -> u32 {
        self.layout_cell_size().0
    }

    /// Sets the cell width without changing its height.
    fn set_layout_cell_width(&mut self, width: u32) {
        let height = self.layout_cell_height();
        self.set_layout_cell_size(width, height);
    }

    /// Returns the cell height.
    fn layout_cell_height(&self) -> u32 {
        self.layout_cell_size().1
    }

    /// Sets the cell height without changing its width.
    fn set_layout_cell_height(&mut self, height: u32) {
        let width = self.layout_cell_width();
        self.set_layout_cell_size(width, height);
    }

    /// Returns the cell x and y offsets.
    fn layout_cell_offset(&self) -> (i32, i32);

    /// Sets the cell x and y offsets.
    fn set_layout_cell_offset(&mut self, x: i32, y: i32);

    /// Returns the cell x offset.
    fn layout_cell_x(&self) -> i32 {
        self.layout_cell_offset().0
    }

    /// Sets the cell x offset without changing its y offset.
    fn set_layout_cell_x(&mut self, x: i32) {
        let y = self.layout_cell_y();
        self.set_layout_cell_offset(x, y);
    }

    /// Returns the cell y offset.
    fn layout_cell_y(&self) -> i32 {
        self.layout_cell_offset().1
    }

    /// Sets the cell y offset without changing its x offset.
    fn set_layout_cell_y(&mut self, y: i32) {
        let x = self.layout_cell_x();
        self.set_layout_cell_offset(x, y);
    }

    /// Returns whether this cell has a parent.
    fn layout_cell_has_parent(&self) -> bool;

    /// Returns the number of direct child cells.
    fn layout_cell_child_count(&self) -> usize;
}

impl LayoutCell for crate::types::layout_cell {
    fn from_layout_cell(
        cell_type: u32,
        flags: i32,
        (width, height): (u32, u32),
        (x, y): (i32, i32),
    ) -> Self {
        Self {
            type_0: cell_type,
            flags,
            sx: width,
            sy: height,
            xoff: x,
            yoff: y,
            ..Self::default()
        }
    }

    fn layout_cell_type(&self) -> u32 {
        self.type_0
    }

    fn set_layout_cell_type(&mut self, cell_type: u32) {
        self.type_0 = cell_type;
    }

    fn layout_cell_flags(&self) -> i32 {
        self.flags
    }

    fn set_layout_cell_flags(&mut self, flags: i32) {
        self.flags = flags;
    }

    fn layout_cell_size(&self) -> (u32, u32) {
        (self.sx, self.sy)
    }

    fn set_layout_cell_size(&mut self, width: u32, height: u32) {
        self.sx = width;
        self.sy = height;
    }

    fn layout_cell_offset(&self) -> (i32, i32) {
        (self.xoff, self.yoff)
    }

    fn set_layout_cell_offset(&mut self, x: i32, y: i32) {
        self.xoff = x;
        self.yoff = y;
    }

    fn layout_cell_has_parent(&self) -> bool {
        self.parent
    }

    fn layout_cell_child_count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::layout_cell;

    #[test]
    fn detached_cell_round_trips_and_updates() {
        let mut cell = layout_cell::from_layout_cell(2, 1, (80, 24), (-3, 4));
        assert_eq!(cell.layout_cell_type(), 2);
        assert_eq!(cell.layout_cell_flags(), 1);
        assert_eq!(cell.layout_cell_size(), (80, 24));
        assert_eq!(cell.layout_cell_offset(), (-3, 4));
        assert!(!cell.layout_cell_has_parent());
        assert_eq!(cell.layout_cell_child_count(), 0);

        cell.set_layout_cell_type(1);
        cell.set_layout_cell_flags(4);
        cell.set_layout_cell_width(120);
        cell.set_layout_cell_height(40);
        cell.set_layout_cell_x(6);
        cell.set_layout_cell_y(-7);
        assert_eq!(cell.layout_cell_type(), 1);
        assert_eq!(cell.layout_cell_flags(), 4);
        assert_eq!(cell.layout_cell_size(), (120, 40));
        assert_eq!(cell.layout_cell_offset(), (6, -7));
    }
}

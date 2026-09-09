//! Borrowed access to variadic-argument backing storage.

/// The offsets and initialized backing regions of a System V variadic cursor.
///
/// The cursor cannot outlive its backing storage.
///
/// ```compile_fail
/// use tmux_c2rs::{VariadicArguments, VariadicCursor};
/// fn escaped() -> VariadicCursor<'static> {
///     let mut bytes = [0; 8];
///     VariadicCursor::from_variadic_arguments(0, 0, Some(&mut bytes), None)
/// }
/// ```
///
/// Mutable access requires an exclusive borrow of the cursor.
///
/// ```compile_fail
/// use tmux_c2rs::{VariadicArguments, VariadicCursor};
/// let mut bytes = [0; 8];
/// let mut cursor = VariadicCursor::from_variadic_arguments(0, 0, Some(&mut bytes), None);
/// let before = cursor.variadic_overflow().unwrap();
/// cursor.variadic_overflow_mut().unwrap()[0] = 1;
/// assert_eq!(before[0], 0);
/// ```
pub trait VariadicArguments<'a> {
    /// Builds a cursor retaining optional, initialized backing regions.
    fn from_variadic_arguments(
        general_offset: u32,
        floating_offset: u32,
        overflow: Option<&'a mut [u8]>,
        registers: Option<&'a mut [u8]>,
    ) -> Self
    where
        Self: Sized;

    fn variadic_general_offset(&self) -> u32;
    fn variadic_floating_offset(&self) -> u32;
    fn variadic_overflow(&self) -> Option<&[u8]>;
    fn variadic_overflow_mut(&mut self) -> Option<&mut [u8]>;
    fn variadic_registers(&self) -> Option<&[u8]>;
    fn variadic_registers_mut(&mut self) -> Option<&mut [u8]>;
}

/// Rust-owned cursor metadata retaining exclusive borrows of its backing regions.
pub struct VariadicCursor<'a> {
    general_offset: u32,
    floating_offset: u32,
    overflow: Option<&'a mut [u8]>,
    registers: Option<&'a mut [u8]>,
}

impl<'a> VariadicArguments<'a> for VariadicCursor<'a> {
    fn from_variadic_arguments(
        general_offset: u32,
        floating_offset: u32,
        overflow: Option<&'a mut [u8]>,
        registers: Option<&'a mut [u8]>,
    ) -> Self {
        Self {
            general_offset,
            floating_offset,
            overflow,
            registers,
        }
    }
    fn variadic_general_offset(&self) -> u32 {
        self.general_offset
    }
    fn variadic_floating_offset(&self) -> u32 {
        self.floating_offset
    }
    fn variadic_overflow(&self) -> Option<&[u8]> {
        self.overflow.as_deref()
    }
    fn variadic_overflow_mut(&mut self) -> Option<&mut [u8]> {
        self.overflow.as_deref_mut()
    }
    fn variadic_registers(&self) -> Option<&[u8]> {
        self.registers.as_deref()
    }
    fn variadic_registers_mut(&mut self) -> Option<&mut [u8]> {
        self.registers.as_deref_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_borrows_and_updates_both_regions() {
        let mut overflow = [1; 8];
        let mut registers = [2; 176];
        {
            let mut value = VariadicCursor::from_variadic_arguments(
                8,
                48,
                Some(&mut overflow),
                Some(&mut registers),
            );
            assert_eq!(value.variadic_general_offset(), 8);
            assert_eq!(value.variadic_floating_offset(), 48);
            assert_eq!(value.variadic_overflow(), Some([1; 8].as_slice()));
            assert_eq!(value.variadic_registers(), Some([2; 176].as_slice()));
            value.variadic_overflow_mut().unwrap()[7] = 3;
            value.variadic_registers_mut().unwrap()[175] = 4;
        }
        assert_eq!(overflow[7], 3);
        assert_eq!(registers[175], 4);
    }

    #[test]
    fn absent_and_empty_regions_remain_distinct() {
        let mut value = VariadicCursor::from_variadic_arguments(0, 0, None, Some(&mut []));
        assert_eq!(value.variadic_overflow(), None);
        assert_eq!(value.variadic_overflow_mut(), None);
        assert_eq!(value.variadic_registers(), Some([].as_slice()));
        assert_eq!(value.variadic_registers_mut().unwrap().len(), 0);
    }
}

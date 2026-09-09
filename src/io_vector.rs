//! Borrowed access to one scatter/gather I/O region.

use std::io::IoSliceMut;

/// One contiguous region used by vectored operating-system I/O.
///
/// ```compile_fail
/// use std::io::IoSliceMut;
/// use tmux_c2rs::IoVector;
/// fn escaped() -> IoSliceMut<'static> {
///     let mut bytes = [0; 8];
///     IoSliceMut::from_io_vector(&mut bytes)
/// }
/// ```
///
/// ```compile_fail
/// use std::io::IoSliceMut;
/// use tmux_c2rs::IoVector;
/// let mut bytes = [0; 8];
/// let mut vector = IoSliceMut::from_io_vector(&mut bytes);
/// let before = vector.io_vector_data();
/// vector.io_vector_data_mut()[0] = 1;
/// assert_eq!(before[0], 0);
/// ```
pub trait IoVector<'a> {
    /// Builds a region retaining an exclusive borrow of its initialized bytes.
    fn from_io_vector(data: &'a mut [u8]) -> Self
    where
        Self: Sized;

    /// Borrows the complete region.
    fn io_vector_data(&self) -> &[u8];

    /// Exclusively borrows the complete region.
    fn io_vector_data_mut(&mut self) -> &mut [u8];

    /// Returns the length of the region in bytes.
    fn io_vector_length(&self) -> usize;
}

impl<'a> IoVector<'a> for IoSliceMut<'a> {
    fn from_io_vector(data: &'a mut [u8]) -> Self {
        Self::new(data)
    }
    fn io_vector_data(&self) -> &[u8] {
        self
    }
    fn io_vector_data_mut(&mut self) -> &mut [u8] {
        self
    }
    fn io_vector_length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_region_reads_and_updates_the_source() {
        let mut bytes = [1, 2, 3];
        {
            let mut vector = IoSliceMut::from_io_vector(&mut bytes);
            assert_eq!(vector.io_vector_data(), &[1, 2, 3]);
            assert_eq!(vector.io_vector_length(), 3);
            vector.io_vector_data_mut()[1] = 7;
        }
        assert_eq!(bytes, [1, 7, 3]);
    }
}

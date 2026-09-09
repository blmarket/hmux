//! Stable access to an imsg payload buffer.

/// The storage bounds, cursor positions, and descriptor of an imsg buffer.
pub trait ImsgDataBuffer {
    /// Builds a buffer from its allocated bytes and cursor state.
    fn from_imsg_data_buffer(
        storage: Vec<u8>,
        max_size: usize,
        write_position: usize,
        read_position: usize,
        file_descriptor: i32,
    ) -> Self
    where
        Self: Sized;

    /// Returns all allocated bytes.
    fn imsg_data_buffer_storage(&self) -> &[u8];

    /// Returns the allocation size.
    fn imsg_data_buffer_size(&self) -> usize;

    /// Resizes the allocation.
    fn resize_imsg_data_buffer(&mut self, size: usize);

    /// Returns the maximum allocation size.
    fn imsg_data_buffer_max_size(&self) -> usize;

    /// Sets the maximum allocation size.
    fn set_imsg_data_buffer_max_size(&mut self, max_size: usize);

    /// Returns the write cursor position.
    fn imsg_data_buffer_write_position(&self) -> usize;

    /// Sets the write cursor position.
    fn set_imsg_data_buffer_write_position(&mut self, position: usize);

    /// Returns the read cursor position.
    fn imsg_data_buffer_read_position(&self) -> usize;

    /// Sets the read cursor position.
    fn set_imsg_data_buffer_read_position(&mut self, position: usize);

    /// Returns the unread bytes.
    fn imsg_data_buffer_readable(&self) -> &[u8];

    /// Returns the attached file descriptor.
    fn imsg_data_buffer_file_descriptor(&self) -> i32;

    /// Sets the attached file descriptor.
    fn set_imsg_data_buffer_file_descriptor(&mut self, file_descriptor: i32);
}

impl ImsgDataBuffer for crate::types::ibuf {
    fn from_imsg_data_buffer(
        storage: Vec<u8>,
        max_size: usize,
        write_position: usize,
        read_position: usize,
        file_descriptor: i32,
    ) -> Self {
        let size = storage.len();
        Self {
            buf: storage.as_slice().into(),
            size,
            max: max_size,
            wpos: write_position,
            rpos: read_position,
            fd: file_descriptor,
            borrowed: false,
        }
    }

    fn imsg_data_buffer_storage(&self) -> &[u8] {
        &self.buf[..self.size]
    }

    fn imsg_data_buffer_size(&self) -> usize {
        self.size
    }

    fn resize_imsg_data_buffer(&mut self, size: usize) {
        self.buf.resize(size, 0);
        self.size = size;
    }

    fn imsg_data_buffer_max_size(&self) -> usize {
        self.max
    }

    fn set_imsg_data_buffer_max_size(&mut self, max_size: usize) {
        self.max = max_size;
    }

    fn imsg_data_buffer_write_position(&self) -> usize {
        self.wpos
    }

    fn set_imsg_data_buffer_write_position(&mut self, position: usize) {
        self.wpos = position;
    }

    fn imsg_data_buffer_read_position(&self) -> usize {
        self.rpos
    }

    fn set_imsg_data_buffer_read_position(&mut self, position: usize) {
        self.rpos = position;
    }

    fn imsg_data_buffer_readable(&self) -> &[u8] {
        &self.buf[self.rpos..self.wpos]
    }

    fn imsg_data_buffer_file_descriptor(&self) -> i32 {
        self.fd
    }

    fn set_imsg_data_buffer_file_descriptor(&mut self, file_descriptor: i32) {
        self.fd = file_descriptor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ibuf;

    #[test]
    fn payload_state_round_trips_and_updates() {
        let mut buffer = ibuf::from_imsg_data_buffer(vec![1, 2, 3, 4], 8, 3, 1, -1);
        assert_eq!(buffer.imsg_data_buffer_storage(), [1, 2, 3, 4]);
        assert_eq!(buffer.imsg_data_buffer_size(), 4);
        assert_eq!(buffer.imsg_data_buffer_max_size(), 8);
        assert_eq!(buffer.imsg_data_buffer_write_position(), 3);
        assert_eq!(buffer.imsg_data_buffer_read_position(), 1);
        assert_eq!(buffer.imsg_data_buffer_readable(), [2, 3]);
        assert_eq!(buffer.imsg_data_buffer_file_descriptor(), -1);

        buffer.set_imsg_data_buffer_max_size(12);
        buffer.resize_imsg_data_buffer(6);
        buffer.set_imsg_data_buffer_write_position(5);
        buffer.set_imsg_data_buffer_read_position(2);
        buffer.set_imsg_data_buffer_file_descriptor(-7);
        assert_eq!(buffer.imsg_data_buffer_storage(), [1, 2, 3, 4, 0, 0]);
        assert_eq!(buffer.imsg_data_buffer_readable(), [3, 4, 0]);
        assert_eq!(buffer.imsg_data_buffer_max_size(), 12);
        assert_eq!(buffer.imsg_data_buffer_file_descriptor(), -7);
    }
}

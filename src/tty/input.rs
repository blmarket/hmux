//! Input storage stays in the terminal while decoding uses its contiguous bytes.
//! Decoded events own their payload before callbacks can replace that storage.
use crate::src::reactor::{evbuffer_pullup, evbuffer_read};
use hmux_buffer::{Buf, SegmentedBuf};

#[derive(Default)]
pub struct TerminalInput {
    buffer: SegmentedBuf,
}

impl TerminalInput {
    pub fn len(&self) -> usize {
        self.buffer.remaining()
    }
    pub fn is_empty(&self) -> bool {
        !self.buffer.has_remaining()
    }
    pub unsafe fn read(&mut self, fd: i32) -> i32 {
        evbuffer_read(&mut self.buffer, fd, -1)
    }
    /// Borrow the buffer's allocation, coalescing segments only when necessary.
    pub fn bytes(&mut self) -> &[u8] {
        evbuffer_pullup(&mut self.buffer, -1).unwrap_or_default()
    }
    pub fn drain(&mut self, count: usize) {
        let count = count.min(self.len());
        self.buffer.advance(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contiguous_input_and_partial_drain_use_the_original_allocation() {
        let bytes = b"a\0\xffbc".to_vec();
        let original = bytes.as_ptr();
        let mut input = TerminalInput {
            buffer: bytes.into(),
        };
        assert_eq!(input.bytes().as_ptr(), original);
        assert_eq!(input.bytes(), b"a\0\xffbc");
        assert_eq!(input.len(), 5);
        input.drain(2);
        assert_eq!(input.bytes().as_ptr(), unsafe { original.add(2) });
        assert_eq!(input.bytes(), b"\xffbc");
        input.drain(usize::MAX);
        assert!(input.is_empty());
        assert!(input.bytes().is_empty());
    }
    #[test]
    fn reads_append_to_remaining_input_and_empty_reads_preserve_storage() {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        let (reader, mut writer) = std::os::unix::net::UnixStream::pair().unwrap();
        let mut input = TerminalInput::default();
        writer.write_all(b"old").unwrap();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 3);
        assert_eq!(input.bytes(), b"old");
        input.drain(1);
        writer.write_all(b"new").unwrap();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 3);
        assert_eq!(input.bytes(), b"ldnew");
        drop(writer);
        let current = input.bytes().as_ptr();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 0);
        assert_eq!(current, input.bytes().as_ptr());
    }
}

//! Input bytes remain published while key callbacks run. Decoder snapshots own
//! their bytes, sharing one allocation while successive keys consume a read.
use crate::src::reactor::{evbuffer_pullup, evbuffer_read};
use hmux_buffer::{Buf, SegmentedBuf};
use std::rc::Rc;

#[derive(Default)]
pub struct TerminalInput {
    buffer: SegmentedBuf,
    cached: Option<InputSnapshot>,
}

#[derive(Clone)]
pub struct InputSnapshot {
    bytes: Rc<[u8]>,
    start: usize,
}

impl AsRef<[u8]> for InputSnapshot {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[self.start..]
    }
}

impl TerminalInput {
    pub fn len(&self) -> usize {
        self.buffer.remaining()
    }
    pub fn is_empty(&self) -> bool {
        !self.buffer.has_remaining()
    }
    pub unsafe fn read(&mut self, fd: i32) -> i32 {
        let read = evbuffer_read(&mut self.buffer, fd, -1);
        if read > 0 {
            self.cached = None;
        }
        read
    }
    pub fn snapshot(&mut self) -> InputSnapshot {
        self.cached
            .get_or_insert_with(|| InputSnapshot {
                bytes: Rc::from(evbuffer_pullup(&mut self.buffer, -1).unwrap_or_default() as &[u8]),
                start: 0,
            })
            .clone()
    }
    pub fn drain(&mut self, count: usize) {
        let count = count.min(self.len());
        self.buffer.advance(count);
        if self.is_empty() {
            self.cached = None;
        } else if let Some(cached) = &mut self.cached {
            cached.start += count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draining_reuses_snapshot_storage_and_retirement_keeps_live_snapshots_valid() {
        let mut input = TerminalInput {
            buffer: b"a\0\xffbc".to_vec().into(),
            cached: None,
        };
        let first = input.snapshot();
        assert_eq!(input.len(), 5);
        input.drain(2);
        let second = input.snapshot();
        assert!(Rc::ptr_eq(&first.bytes, &second.bytes));
        assert_eq!(second.as_ref(), b"\xffbc");
        input.drain(usize::MAX);
        assert!(input.is_empty());
        assert_eq!(first.as_ref(), b"a\0\xffbc");
        assert_eq!(second.as_ref(), b"\xffbc");
        assert!(input.snapshot().as_ref().is_empty());
    }
    #[test]
    fn successful_reads_publish_new_bytes_without_changing_older_snapshots() {
        use std::io::Write;
        use std::os::fd::AsRawFd;
        let (reader, mut writer) = std::os::unix::net::UnixStream::pair().unwrap();
        let mut input = TerminalInput::default();
        writer.write_all(b"old").unwrap();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 3);
        let first = input.snapshot();
        input.drain(1);
        writer.write_all(b"new").unwrap();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 3);
        assert_eq!(input.snapshot().as_ref(), b"ldnew");
        assert_eq!(first.as_ref(), b"old");
        drop(writer);
        let current = input.snapshot();
        assert_eq!(unsafe { input.read(reader.as_raw_fd()) }, 0);
        assert!(Rc::ptr_eq(&current.bytes, &input.snapshot().bytes));
    }
}

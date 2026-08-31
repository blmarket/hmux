use bytes::{Buf as BytesBuf, BufMut as BytesBufMut, Bytes, BytesMut};
use bytes_utils::SegmentedBuf;
use std::io::{self, IoSlice};
use std::os::fd::RawFd;

const TAIL_CAPACITY: usize = 16 * 1024;
const MAX_IOV: usize = 64;

#[derive(Clone, Debug, Default)]
pub struct Buf {
    segments: SegmentedBuf<Bytes>,
    tail: BytesMut,
}

impl Buf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: SegmentedBuf::new(),
            tail: BytesMut::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.segments.remaining() + self.tail.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn append(&mut self, data: &[u8]) {
        let mut remaining = data;
        while !remaining.is_empty() {
            if self.tail.capacity() == self.tail.len() {
                self.tail.reserve(TAIL_CAPACITY);
            }
            let available = self.tail.capacity() - self.tail.len();
            let count = available.min(remaining.len());
            self.tail.extend_from_slice(&remaining[..count]);
            remaining = &remaining[count..];
        }
    }

    pub fn append_bytes(&mut self, data: Bytes) {
        if data.is_empty() {
            return;
        }
        self.seal_tail();
        self.segments.push(data);
    }

    pub fn append_buf(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }
        self.seal_tail();
        other.seal_tail();
        let segments = std::mem::take(&mut other.segments).into_inner();
        self.segments.extend(segments);
    }

    pub fn drain(&mut self, count: usize) {
        self.advance(count.min(self.len()));
    }

    pub fn clear(&mut self) {
        self.segments = SegmentedBuf::new();
        self.tail.clear();
    }

    pub fn copy_to_bytes(&mut self, count: usize) -> Bytes {
        let count = count.min(self.len());
        if count == 0 {
            return Bytes::new();
        }
        if self.segments.remaining() == 0 {
            return self.tail.split_to(count).freeze();
        }
        if count <= self.segments.remaining() {
            return self.segments.copy_to_bytes(count);
        }

        let ready = self.segments.remaining();
        let mut result = BytesMut::with_capacity(count);
        result.extend_from_slice(&self.segments.copy_to_bytes(ready));
        result.extend_from_slice(&self.tail.split_to(count - ready));
        result.freeze()
    }

    pub fn pullup(&mut self, count: usize) -> &[u8] {
        let count = count.min(self.len());
        if count == 0 {
            return &[];
        }
        if self.segments.remaining() == 0 {
            return &self.tail[..count];
        }
        if self.segments.chunk().len() >= count {
            return &self.segments.chunk()[..count];
        }

        self.seal_tail();
        let total = self.segments.remaining();
        let contiguous = self.segments.copy_to_bytes(total);
        self.segments.push(contiguous);
        &self.segments.chunk()[..count]
    }

    pub fn as_slice(&mut self) -> &[u8] {
        self.pullup(self.len())
    }

    pub fn read_line(&mut self) -> Option<Bytes> {
        let total = self.len();
        let data = self.pullup(total);
        let end = data.iter().position(|byte| *byte == b'\n')?;
        let line_len = usize::from(end > 0 && data[end - 1] == b'\r');
        let result = self.copy_to_bytes(end - line_len);
        self.advance(usize::from(line_len != 0) + 1);
        Some(result)
    }

    pub fn read_from_fd(&mut self, fd: RawFd, max_read: usize) -> io::Result<usize> {
        if max_read == 0 {
            return Ok(0);
        }
        let read_limit = max_read.min(64 * 1024);
        if self.tail.capacity() - self.tail.len() < read_limit {
            self.tail.reserve(read_limit);
        }
        let spare = self.tail.spare_capacity_mut();
        let count = read_limit.min(spare.len());
        let result = unsafe { libc::read(fd, spare.as_mut_ptr().cast::<libc::c_void>(), count) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        unsafe { BytesBufMut::advance_mut(&mut self.tail, result as usize) };
        Ok(result as usize)
    }

    pub fn write_to_fd(&mut self, fd: RawFd) -> io::Result<usize> {
        self.seal_tail();
        if !self.segments.has_remaining() {
            return Ok(0);
        }
        let mut slices = [IoSlice::new(&[]); MAX_IOV];
        let count = self.segments.chunks_vectored(&mut slices);
        let result = unsafe { libc::writev(fd, slices.as_ptr().cast(), count as libc::c_int) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        self.segments.advance(result as usize);
        Ok(result as usize)
    }

    pub fn advance(&mut self, mut count: usize) {
        assert!(count <= self.len());
        let from_segments = count.min(self.segments.remaining());
        self.segments.advance(from_segments);
        count -= from_segments;
        if count != 0 {
            BytesBuf::advance(&mut self.tail, count);
        }
    }

    fn seal_tail(&mut self) {
        if !self.tail.is_empty() {
            self.segments.push(self.tail.split().freeze());
        }
    }
}

impl bytes::Buf for Buf {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn chunk(&self) -> &[u8] {
        if self.segments.has_remaining() {
            self.segments.chunk()
        } else {
            &self.tail
        }
    }

    fn advance(&mut self, count: usize) {
        Self::advance(self, count);
    }

    fn copy_to_bytes(&mut self, count: usize) -> Bytes {
        Self::copy_to_bytes(self, count)
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        let mut count = self.segments.chunks_vectored(dst);
        if count < dst.len() && !self.tail.is_empty() {
            dst[count] = IoSlice::new(&self.tail);
            count += 1;
        }
        count
    }
}

unsafe impl bytes::BufMut for Buf {
    fn remaining_mut(&self) -> usize {
        isize::MAX as usize - self.len()
    }

    unsafe fn advance_mut(&mut self, count: usize) {
        let available = self.tail.capacity() - self.tail.len();
        assert!(count <= available);
        unsafe { BytesBufMut::advance_mut(&mut self.tail, count) };
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        if self.tail.capacity() == self.tail.len() {
            self.tail.reserve(TAIL_CAPACITY);
        }
        self.tail.spare_capacity_mut().into()
    }
}

#[cfg(test)]
#[path = "../tests/test_reactor_buffer.rs"]
mod tests;

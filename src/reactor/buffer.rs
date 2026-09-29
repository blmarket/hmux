//! Encapsulated byte storage with mutation notifications for stream scheduling.
use crate::src::shared::abi::{size_t, ssize_t};
pub use hmux_buffer::LineEnding;
use hmux_buffer::{Buf, BufMut, Buffer, SegmentedBuf as ByteBuffer};
use std::ffi::{c_int, c_void};
use std::rc::Rc;

/// The only application interface to event-buffer storage.
pub trait EventBuffer {
    fn new() -> Box<Self>;
    /// Construct a buffer whose mutations notify its owning stream.
    fn with_notify(notify: Rc<dyn Fn()>) -> Box<Self>;
    #[cfg(test)]
    fn segment_count(&self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn add_slice(&mut self, bytes: &[u8]);
    /// # Safety
    /// `data` must be readable for `len` bytes when `len` is nonzero.
    unsafe fn add_raw(&mut self, data: *const c_void, len: size_t) -> c_int;
    fn drain(&mut self, len: usize) -> c_int;
    /// Borrow a contiguous prefix, or all bytes for a negative size.
    /// Empty buffers and oversized requests return `None`.
    fn pullup(&mut self, size: ssize_t) -> Option<&mut [u8]>;
    /// Read up to `limit` bytes (64 KiB for a negative limit) from a descriptor.
    unsafe fn read_fd(&mut self, fd: c_int, limit: c_int) -> c_int;
    unsafe fn write_fd(&mut self, fd: c_int) -> c_int;
    /// Consume a complete line and return its bytes with a trailing NUL.
    fn read_line(&mut self, ending: LineEnding) -> Option<Vec<u8>>;
    fn add_formatted(
        &mut self,
        write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    ) -> c_int;
    /// Move all source segments and notify both buffers.
    fn append(&mut self, source: &mut Self);
}

#[derive(Default)]
pub struct evbuffer {
    bytes: ByteBuffer,
    notify: Option<Rc<dyn Fn()>>,
}

impl EventBuffer for evbuffer {
    fn new() -> Box<Self> {
        Box::default()
    }
    fn with_notify(notify: Rc<dyn Fn()>) -> Box<Self> {
        Box::new(Self {
            bytes: ByteBuffer::default(),
            notify: Some(notify),
        })
    }
    #[cfg(test)]
    fn segment_count(&self) -> usize {
        self.bytes.chunks().count()
    }
    fn len(&self) -> usize {
        self.bytes.remaining()
    }
    fn is_empty(&self) -> bool {
        !self.bytes.has_remaining()
    }
    fn add_slice(&mut self, bytes: &[u8]) {
        self.bytes.put_slice(bytes);
        if let Some(notify) = &self.notify {
            notify();
        }
    }
    unsafe fn add_raw(&mut self, data: *const c_void, len: size_t) -> c_int {
        self.add_slice(if len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(data.cast(), len)
        });
        0
    }
    fn drain(&mut self, len: usize) -> c_int {
        self.bytes.advance(len.min(self.len()));
        if let Some(notify) = &self.notify {
            notify();
        }
        0
    }
    fn pullup(&mut self, size: ssize_t) -> Option<&mut [u8]> {
        if self.is_empty() || (size >= 0 && size as usize > self.len()) {
            return None;
        }
        let count = if size < 0 { self.len() } else { size as usize };
        self.bytes.pullup(count)
    }
    unsafe fn read_fd(&mut self, fd: c_int, limit: c_int) -> c_int {
        let count = if limit < 0 {
            65536
        } else {
            (limit as usize).min(65536)
        };
        let mut bytes = Vec::<u8>::with_capacity(count);
        let n = libc::read(fd, bytes.as_mut_ptr().cast(), count);
        if n > 0 {
            bytes.set_len(n as usize);
            self.bytes.put(ByteBuffer::from(bytes));
            if let Some(notify) = &self.notify {
                notify();
            }
        }
        n as c_int
    }
    unsafe fn write_fd(&mut self, fd: c_int) -> c_int {
        let mut chunks = [libc::iovec {
            iov_base: std::ptr::null_mut(),
            iov_len: 0,
        }; 64];
        let mut count = 0;
        let mut remaining = 65536;
        for chunk in self.bytes.chunks().take(64) {
            let len = chunk.len().min(remaining);
            chunks[count] = libc::iovec {
                iov_base: chunk.as_ptr() as *mut c_void,
                iov_len: len,
            };
            count += 1;
            remaining -= len;
            if remaining == 0 {
                break;
            }
        }
        let n = libc::writev(fd, chunks.as_ptr(), count as c_int);
        if n > 0 {
            self.drain(n as usize);
        }
        n as c_int
    }
    fn read_line(&mut self, ending: LineEnding) -> Option<Vec<u8>> {
        let mut line = self.bytes.read_line(ending)?;
        line.push(0);
        if let Some(notify) = &self.notify {
            notify();
        }
        Some(line)
    }
    fn add_formatted(
        &mut self,
        write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    ) -> c_int {
        let Some(mut formatted) = format_buffer(write) else {
            return -1;
        };
        let count = formatted.remaining();
        self.bytes.append(&mut formatted);
        if let Some(notify) = &self.notify {
            notify();
        }
        count as c_int
    }
    fn append(&mut self, source: &mut Self) {
        self.bytes.append(&mut source.bytes);
        if let Some(notify) = &source.notify {
            notify();
        }
        if let Some(notify) = &self.notify {
            notify();
        }
    }
}

/// Keep a trailing NUL in spare capacity, excluded from the readable payload.
/// Moving this segment with append preserves its allocation and capacity.
fn format_buffer(
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> Option<ByteBuffer> {
    let mut bytes = crate::src::format::bytes::format_bytes_with(write).ok()?;
    if bytes.len() > c_int::MAX as usize {
        return None;
    }
    bytes.try_reserve(1).ok()?;
    bytes.push(0);
    bytes.pop();
    Some(ByteBuffer::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::format::bytes::write_cstr;
    use std::ffi::CString;

    #[test]
    fn pullup_borrows_prefix_and_preserves_request_bounds() {
        let mut buffer = evbuffer::new();
        assert!(buffer.pullup(-1).is_none());
        buffer.add_slice(b"abc");
        buffer.add_slice(b"def");

        assert!(buffer.pullup(7).is_none());
        let prefix = buffer.pullup(4).unwrap();
        assert_eq!(prefix, b"abcd");
        prefix[3] = b'D';

        assert_eq!(buffer.len(), 6);
        assert_eq!(buffer.pullup(-1).unwrap(), b"abcDef");
        buffer.drain(4);
        assert_eq!(buffer.pullup(-1).unwrap(), b"ef");
        buffer.drain(usize::MAX);
        assert!(buffer.pullup(0).is_none());
    }

    #[test]
    fn transfer_preserves_segments_and_notifies_both_owners() {
        use std::cell::Cell;
        let source_wakes = Rc::new(Cell::new(0));
        let destination_wakes = Rc::new(Cell::new(0));
        let notified = source_wakes.clone();
        let mut source = evbuffer::with_notify(Rc::new(move || notified.set(notified.get() + 1)));
        let notified = destination_wakes.clone();
        let mut destination =
            evbuffer::with_notify(Rc::new(move || notified.set(notified.get() + 1)));
        source.add_slice(b"first");
        source.add_formatted(|out| out.write_all(b"second\n"));
        assert_eq!(source_wakes.get(), 2);
        let pointer = source.pullup(5).unwrap().as_ptr();
        destination.append(&mut source);
        assert!(source.is_empty());
        assert_eq!(source_wakes.get(), 3);
        assert_eq!(destination_wakes.get(), 1);
        assert_eq!(destination.segment_count(), 2);
        assert_eq!(destination.pullup(5).unwrap().as_ptr(), pointer);
        assert_eq!(
            destination.read_line(LineEnding::Lf).unwrap(),
            b"firstsecond\0"
        );
        assert_eq!(destination_wakes.get(), 2);
        assert!(destination.read_line(LineEnding::Lf).is_none());
        assert_eq!(destination_wakes.get(), 2);
        assert_eq!(
            destination.add_formatted(|_| Err(std::io::Error::other("failed"))),
            -1
        );
        assert_eq!(destination_wakes.get(), 2);
        destination.drain(usize::MAX);
        assert_eq!(destination_wakes.get(), 3);
    }

    fn append_formatted(
        destination: &mut ByteBuffer,
        write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    ) -> c_int {
        let Some(mut formatted) = format_buffer(write) else {
            return -1;
        };
        let count = formatted.remaining();
        let pointer = formatted.chunk().as_ptr();
        destination.append(&mut formatted);
        assert_eq!(formatted.remaining(), 0);
        if count != 0 {
            assert_eq!(destination.chunks().next_back().unwrap().as_ptr(), pointer);
        }
        count as c_int
    }

    #[test]
    fn formatted_storage_survives_append_and_terminator_commit() {
        for size in [1, 1023, 1024, 4095, 4096, 4097, 40076, 100_000] {
            let text = CString::new("x".repeat(size)).unwrap();
            let mut buffer = ByteBuffer::default();
            assert_eq!(
                unsafe {
                    append_formatted(&mut buffer, |out| {
                        write_cstr(out, text.as_ptr())?;
                        write!(out, ":{}:{}", (-7i32) as i32, (42usize) as usize)
                    })
                },
                (size + 6) as c_int
            );
            assert_eq!(buffer.chunks().len(), 1);
            let length = buffer.remaining();
            let pointer = buffer.pullup(length).unwrap().as_ptr();
            assert_eq!(
                unsafe { std::ffi::CStr::from_ptr(pointer.cast()) }.to_bytes(),
                buffer.chunk()
            );
            assert_eq!(
                buffer.chunk(),
                format!("{}:-7:42", "x".repeat(size)).as_bytes()
            );
            buffer.put_slice(b"\0");
            assert_eq!(buffer.chunks().len(), 1);
            assert_eq!(buffer.chunk().as_ptr(), pointer);
            assert_eq!(buffer.remaining(), length + 1);
            assert_eq!(buffer.chunk()[length], 0);
        }
    }

    #[test]
    fn formatting_preserves_prefix_empty_output_and_embedded_nul() {
        let mut buffer = ByteBuffer::from(b"prefix".to_vec());
        unsafe {
            assert_eq!(
                append_formatted(&mut buffer, |out| { write_cstr(out, c"".as_ptr()) }),
                0
            );
            assert_eq!(buffer.remaining(), 6);
            assert_eq!(
                append_formatted(&mut buffer, |out| {
                    out.write_all(&[(0i32) as u8])?;
                    write!(out, ":{}", (7i32) as i32)
                }),
                3
            );
        }
        assert_eq!(
            buffer.chunks().flatten().copied().collect::<Vec<_>>(),
            b"prefix\0:7"
        );
    }
}

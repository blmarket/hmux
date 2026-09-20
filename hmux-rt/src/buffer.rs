//! Contiguous, binary-safe storage for buffered I/O.
use std::io::{self, Read, Write};

/// Terminator handling for binary-safe line extraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    /// Consume a run of carriage returns and newlines.
    Any,
    /// Consume LF, removing an optional preceding CR from the returned line.
    CrLf,
    /// Require the two-byte CR LF sequence.
    CrLfStrict,
    /// Consume exactly one LF.
    Lf,
    /// Consume exactly one NUL.
    Nul,
    /// Consume CR or LF, plus an adjacent terminator of the opposite kind.
    Legacy,
}

/// Byte storage with cheap prefix drains and contiguous access.
///
/// This type has no runtime or descriptor ownership. Use it independently or
/// as storage for a stream adapter. Raw pointers obtained from a borrowed slice
/// must not survive mutation or destruction.
#[derive(Default, Debug)]
pub struct ByteBuffer {
    bytes: Vec<u8>,
    start: usize,
}

impl ByteBuffer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.bytes.len() - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[self.start..]
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[self.start..]
    }
    pub fn append(&mut self, data: &[u8]) {
        if self.start > 0 && self.bytes.capacity() - self.bytes.len() < data.len() {
            self.bytes.copy_within(self.start.., 0);
            self.bytes.truncate(self.len());
            self.start = 0;
        }
        self.bytes.extend_from_slice(data);
    }
    pub fn drain(&mut self, count: usize) {
        self.start += count.min(self.len());
        if self.start == self.bytes.len() {
            self.bytes.clear();
            self.start = 0;
        }
    }
    pub fn transfer(&mut self, other: &mut Self) {
        self.append(other.as_slice());
        other.drain(other.len());
    }
    /// Read one bounded chunk, preserving storage on EOF or error.
    /// Interrupted and would-block errors are returned for the caller to handle.
    pub fn read_from(&mut self, reader: &mut impl Read, limit: usize) -> io::Result<usize> {
        if limit == 0 {
            return Ok(0);
        }
        let mut bytes = vec![0; limit.min(64 * 1024)];
        let count = reader.read(&mut bytes)?;
        self.append(&bytes[..count]);
        Ok(count)
    }

    /// Write once and drain only bytes accepted by the writer.
    pub fn write_to(&mut self, writer: &mut impl Write) -> io::Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }
        let count = writer.write(self.as_slice())?;
        self.drain(count);
        Ok(count)
    }

    /// Read a bounded chunk from a descriptor, retrying interruptions.
    ///
    /// # Safety
    /// The descriptor must be open and readable throughout this call.
    pub unsafe fn read_from_fd(
        &mut self,
        fd: std::os::fd::RawFd,
        limit: usize,
    ) -> io::Result<usize> {
        let mut bytes = vec![0u8; limit.min(64 * 1024)];
        loop {
            let n = unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
            if n >= 0 {
                self.append(&bytes[..n as usize]);
                return Ok(n as usize);
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }

    /// Write a bounded chunk and drain only accepted bytes.
    ///
    /// # Safety
    /// The descriptor must be open and writable throughout this call.
    pub unsafe fn write_to_fd(&mut self, fd: std::os::fd::RawFd) -> io::Result<usize> {
        loop {
            let bytes = self.as_slice();
            let n = unsafe {
                libc::write(
                    fd,
                    bytes.as_ptr().cast(),
                    bytes.len().min(i32::MAX as usize),
                )
            };
            if n >= 0 {
                self.drain(n as usize);
                return Ok(n as usize);
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }

    /// Return the line and consume its terminator, leaving incomplete lines.
    pub fn read_line(&mut self, style: LineEnding) -> Option<Vec<u8>> {
        let bytes = self.as_slice();
        let (end, consumed) = match style {
            LineEnding::Nul => {
                let p = bytes.iter().position(|&b| b == 0)?;
                (p, p + 1)
            }
            LineEnding::Lf => {
                let p = bytes.iter().position(|&b| b == b'\n')?;
                (p, p + 1)
            }
            LineEnding::CrLfStrict => {
                let p = bytes.windows(2).position(|s| s == b"\r\n")?;
                (p, p + 2)
            }
            LineEnding::CrLf => {
                let p = bytes.iter().position(|&b| b == b'\n')?;
                (
                    if p > 0 && bytes[p - 1] == b'\r' {
                        p - 1
                    } else {
                        p
                    },
                    p + 1,
                )
            }
            LineEnding::Any => {
                let p = bytes.iter().position(|&b| b == b'\r' || b == b'\n')?;
                let n = bytes[p..]
                    .iter()
                    .take_while(|&&b| b == b'\r' || b == b'\n')
                    .count();
                (p, p + n)
            }
            LineEnding::Legacy => {
                let p = bytes.iter().position(|&b| b == b'\r' || b == b'\n')?;
                let extra = usize::from(
                    bytes
                        .get(p + 1)
                        .is_some_and(|&b| (b == b'\r' || b == b'\n') && b != bytes[p]),
                );
                (p, p + 1 + extra)
            }
        };
        let line = bytes[..end].to_vec();
        self.drain(consumed);
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_compaction_and_transfer() {
        let mut a = ByteBuffer::new();
        a.append(b"abc\0def");
        a.drain(3);
        a.append(&vec![b'x'; 100_000]);
        assert_eq!(&a.as_slice()[..4], b"\0def");
        let mut b = ByteBuffer::new();
        b.transfer(&mut a);
        assert!(a.is_empty());
        assert_eq!(b.len(), 100_004);
        b.drain(usize::MAX);
        assert!(b.is_empty());
    }
    #[test]
    fn split_endings_and_binary_lines() {
        let mut b = ByteBuffer::new();
        b.append(b"one\r");
        assert_eq!(b.read_line(LineEnding::CrLfStrict), None);
        b.append(b"\n\n\0two\npartial");
        assert_eq!(b.read_line(LineEnding::CrLfStrict), Some(b"one".to_vec()));
        assert_eq!(b.read_line(LineEnding::Lf), Some(vec![]));
        assert_eq!(b.read_line(LineEnding::Lf), Some(b"\0two".to_vec()));
        assert_eq!(b.read_line(LineEnding::Lf), None);
        assert_eq!(b.as_slice(), b"partial");
    }
}

#[cfg(test)]
mod io_tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn short_write_and_would_block_preserve_unconsumed_bytes() {
        let mut b = ByteBuffer::new();
        b.append(b"abcdef");
        let mut output = [0; 2];
        assert_eq!(b.write_to(&mut output.as_mut_slice()).unwrap(), 2);
        assert_eq!(&output, b"ab");
        assert_eq!(b.as_slice(), b"cdef");
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        assert_eq!(
            b.read_from(&mut reader, 10).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(b.as_slice(), b"cdef");
        writer.write_all(b"xy").unwrap();
        assert_eq!(b.read_from(&mut reader, 1).unwrap(), 1);
        assert_eq!(b.as_slice(), b"cdefx");
        assert_eq!(b.read_from(&mut reader, 10).unwrap(), 1);
        drop(writer);
        assert_eq!(b.read_from(&mut reader, 10).unwrap(), 0);
    }

    #[test]
    fn line_policies_remain_distinct() {
        for (style, line, rest) in [
            (LineEnding::Any, &b"a"[..], &b"b\0c\n"[..]),
            (LineEnding::Legacy, &b"a"[..], &b"\nb\0c\n"[..]),
            (LineEnding::CrLf, &b"a"[..], &b"\nb\0c\n"[..]),
            (LineEnding::CrLfStrict, &b"a"[..], &b"\nb\0c\n"[..]),
            (LineEnding::Lf, &b"a\r"[..], &b"\nb\0c\n"[..]),
            (LineEnding::Nul, &b"a\r\n\nb"[..], &b"c\n"[..]),
        ] {
            let mut b = ByteBuffer::new();
            b.append(b"a\r\n\nb\0c\n");
            assert_eq!(b.read_line(style).unwrap(), line);
            assert_eq!(b.as_slice(), rest);
        }
    }
}

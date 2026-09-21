//! One-way byte-stream capabilities backed by a single I/O registration.

use std::io;
use std::os::fd::OwnedFd;
use std::rc::Rc;

use crate::{AsyncRead, AsyncWrite, Handle};

/// An async reader for a nonblocking byte-stream descriptor, such as a socket
/// or pipe. Datagram and message-oriented descriptors are not supported.
///
/// Reads run only while their future is polled. Dropping a pending read consumes
/// no bytes. The caller must keep the descriptor nonblocking and should avoid
/// reading through other aliases. This adapter uses the runtime's local wakers.
///
/// A reader cannot be passed to code requiring write access:
/// ```compile_fail
/// use hmux_rt::{AsyncWrite, mio, stream::Reader};
/// fn needs_writer<T: AsyncWrite>() {}
/// needs_writer::<Reader<mio::Io>>();
/// ```
pub struct Reader<I: AsyncRead> {
    source: I,
}

impl<I: AsyncRead> Reader<I> {
    /// Register a nonblocking byte stream with the runtime.
    ///
    /// The descriptor's flags are not changed. Registration errors, including
    /// an already registered descriptor, are returned to the caller.
    pub fn new<H: Handle<Io = I>>(handle: &H, fd: Rc<OwnedFd>) -> io::Result<Self> {
        let source = handle.io(fd)?;
        Ok(Self { source })
    }

    /// Restrict an existing reader to read-only access without registering again.
    pub fn from_io(source: I) -> Self {
        Self { source }
    }

    /// Await an owned chunk of at most `max_bytes`, or `None` at EOF.
    ///
    /// `max_bytes` must be nonzero. Chunks reflect individual reads, not message
    /// boundaries. No background task or read-ahead buffer is created.
    pub async fn read_chunk(&mut self, max_bytes: usize) -> io::Result<Option<Vec<u8>>> {
        if max_bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chunk size must be nonzero",
            ));
        }
        let mut bytes = vec![0; max_bytes];
        let count = self.source.read(&mut bytes).await?;
        if count == 0 {
            return Ok(None);
        }
        bytes.truncate(count);
        Ok(Some(bytes))
    }
}

impl<I: AsyncRead> AsyncRead for Reader<I> {
    async fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.source.read(buffer).await
    }
}

/// A write-only view of a nonblocking byte stream, such as a child's stdin pipe.
/// This wrapper implements [`AsyncWrite`] only and adds no buffering.
/// The host supplies a writable endpoint and retains SIGPIPE policy.
///
/// A writer cannot be passed to code requiring read access:
/// ```compile_fail
/// use hmux_rt::{AsyncRead, mio, stream::Writer};
/// fn needs_reader<T: AsyncRead>() {}
/// needs_reader::<Writer<mio::Io>>();
/// ```
pub struct Writer<I: AsyncWrite> {
    source: I,
}

impl<I: AsyncWrite> Writer<I> {
    /// Register a writable nonblocking byte stream once, without changing flags.
    pub fn new<H: Handle<Io = I>>(handle: &H, fd: Rc<OwnedFd>) -> io::Result<Self> {
        Ok(Self {
            source: handle.io(fd)?,
        })
    }

    /// Restrict an existing writer to write-only access without registering again.
    pub fn from_io(source: I) -> Self {
        Self { source }
    }
}

impl<I: AsyncWrite> AsyncWrite for Writer<I> {
    async fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        self.source.write(buffer).await
    }
}

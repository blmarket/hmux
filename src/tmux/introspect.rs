//! Introspection tap: logging decorators over [`FrameReader`]/[`FrameWriter`].
//!
//! Wrapping either half logs each frame as it passes, giving a control-plane
//! trace for free. This is the only place hmux "sees" the conversation —
//! keystrokes and screen output travel over the passed tty fd, not through
//! frames (design.md "Scope & limits").

use std::{fmt, io};

use tracing::info;

use super::message::{Frame, Message};
use super::traits::{
    FrameReader, FrameWriter, NonblockingFrameReader, NonblockingFrameWriter, WriteQueueFull,
};

/// Direction label for logged frames.
#[derive(Clone, Copy)]
pub enum Direction {
    /// client → server
    ClientToServer,
    /// server → client
    ServerToClient,
}

impl Direction {
    fn arrow(self) -> &'static str {
        match self {
            Direction::ClientToServer => "c->s",
            Direction::ServerToClient => "s->c",
        }
    }
}

/// Logs each received frame, then hands it through unchanged.
pub struct LoggingReader<R> {
    inner: R,
    dir: Direction,
}

impl<R> LoggingReader<R> {
    pub fn new(inner: R, dir: Direction) -> Self {
        LoggingReader { inner, dir }
    }
}

impl<R: FrameReader> FrameReader for LoggingReader<R> {
    fn recv(&mut self) -> io::Result<Frame> {
        let frame = self.inner.recv()?;
        log_frame(self.dir, &frame);
        Ok(frame)
    }
}

impl<R: NonblockingFrameReader> NonblockingFrameReader for LoggingReader<R> {
    fn try_recv(&mut self) -> io::Result<Frame> {
        let frame = self.inner.try_recv()?;
        log_frame(self.dir, &frame);
        Ok(frame)
    }
}

/// Logs each frame before sending it through unchanged.
pub struct LoggingWriter<W> {
    inner: W,
    dir: Direction,
}

impl<W> LoggingWriter<W> {
    pub fn new(inner: W, dir: Direction) -> Self {
        LoggingWriter { inner, dir }
    }
}

impl<W: FrameWriter> FrameWriter for LoggingWriter<W> {
    fn send(&mut self, frame: Frame) -> io::Result<()> {
        log_frame(self.dir, &frame);
        self.inner.send(frame)
    }
}

impl<W> NonblockingFrameWriter for LoggingWriter<W>
where
    W: NonblockingFrameWriter<Frame = Frame>,
{
    type Frame = Frame;

    fn try_queue(&mut self, frame: Self::Frame) -> Result<(), WriteQueueFull<Self::Frame>> {
        log_frame(self.dir, &frame);
        self.inner.try_queue(frame)
    }

    fn try_flush(&mut self) -> io::Result<()> {
        self.inner.try_flush()
    }

    fn has_pending(&self) -> bool {
        self.inner.has_pending()
    }
}

pub(crate) fn log_frame(dir: Direction, frame: &Frame) {
    info!(
        dir = dir.arrow(),
        msg = frame.msg.name(),
        version = frame.version,
        has_fd = frame.fd.is_some(),
        "{}",
        LogMessage(&frame.msg),
    );
}

struct LogMessage<'a>(&'a Message);

impl fmt::Display for LogMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Message::ReadOpen { stream, fd, path } => {
                write!(f, "ReadOpen {{ stream: {stream}, fd: {fd}, ")?;
                fmt_bytes_field(f, "path", path)?;
                write!(f, " }}")
            }
            Message::WriteOpen {
                stream,
                fd,
                flags,
                path,
            } => {
                write!(
                    f,
                    "WriteOpen {{ stream: {stream}, fd: {fd}, flags: {flags}, "
                )?;
                fmt_bytes_field(f, "path", path)?;
                write!(f, " }}")
            }
            Message::Write { stream, data } => {
                write!(f, "Write {{ stream: {stream}, ")?;
                fmt_bytes_field(f, "data", data)?;
                write!(f, " }}")
            }
            Message::Status { revision, body } => {
                write!(f, "Status {{ revision: {revision}, ")?;
                fmt_bytes_field(f, "body", body)?;
                write!(f, " }}")
            }
            Message::Unknown { type_, payload } => {
                write!(f, "Unknown {{ type_: {type_}, ")?;
                fmt_bytes_field(f, "payload", payload)?;
                write!(f, " }}")
            }
            msg => write!(f, "{msg:?}"),
        }
    }
}

fn fmt_bytes_field(f: &mut fmt::Formatter<'_>, name: &str, bytes: &[u8]) -> fmt::Result {
    write!(
        f,
        "{name}_len: {}, {name}: {:?}",
        bytes.len(),
        String::from_utf8_lossy(bytes).as_ref()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_data_logs_as_lossy_utf8_with_length() {
        let msg = Message::Write {
            stream: 1,
            data: b"%0\t/home/hun/srv/d1\n".to_vec(),
        };

        let rendered = LogMessage(&msg).to_string();

        assert_eq!(
            rendered,
            "Write { stream: 1, data_len: 20, data: \"%0\\t/home/hun/srv/d1\\n\" }"
        );
        assert!(!rendered.contains("[37,"));
    }

    #[test]
    fn status_body_logs_as_lossy_utf8_with_length() {
        let msg = Message::Status {
            revision: 24,
            body: b"%1\t0\t1\t0\tclaude\tworking\n".to_vec(),
        };

        let rendered = LogMessage(&msg).to_string();

        assert_eq!(
            rendered,
            "Status { revision: 24, body_len: 24, body: \"%1\\t0\\t1\\t0\\tclaude\\tworking\\n\" }"
        );
        assert!(!rendered.contains("[37,"));
    }

    #[test]
    fn invalid_utf8_is_lossy_but_not_decimal_bytes() {
        let msg = Message::Write {
            stream: 2,
            data: vec![b'o', b'k', 0xff],
        };

        let rendered = LogMessage(&msg).to_string();

        assert_eq!(
            rendered,
            format!(
                "Write {{ stream: 2, data_len: 3, data: \"ok{}\" }}",
                char::REPLACEMENT_CHARACTER
            )
        );
        assert!(!rendered.contains("[111,"));
    }
}

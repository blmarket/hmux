//! Introspection tap: logging for frames as they cross a connection.
//!
//! Logging a frame on the way past gives a control-plane trace for free. This
//! is the only place hmux "sees" the conversation — keystrokes and screen
//! output travel over the passed tty fd, not through frames.

use std::fmt;

use tracing::info;

use super::message::{Frame, Message};

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

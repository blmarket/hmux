//! Stable access to the fixed headers used by the tmux wire protocol.

use core::ffi::c_int;

/// The fixed header preceding a packed command argument vector.
pub trait CommandMessage {
    /// Builds a command header.
    fn from_command_message(argument_count: c_int) -> Self
    where
        Self: Sized;

    /// Returns the number of packed arguments following the header.
    fn command_argument_count(&self) -> c_int;
}

/// A wire message that identifies a file stream.
pub trait StreamMessage {
    /// Builds a message for a file stream.
    fn from_stream_message(stream: c_int) -> Self
    where
        Self: Sized;

    /// Returns the file stream identifier.
    fn message_stream(&self) -> c_int;
}

/// The fixed header opening a stream for reading.
pub trait ReadOpenMessage {
    /// Builds a read-open header.
    fn from_read_open_message(stream: c_int, fd: c_int) -> Self
    where
        Self: Sized;

    /// Returns the file stream identifier.
    fn read_open_stream(&self) -> c_int;

    /// Returns the file descriptor requested for the stream.
    fn read_open_fd(&self) -> c_int;
}

/// The fixed header opening a stream for writing.
pub trait WriteOpenMessage {
    /// Builds a write-open header.
    fn from_write_open_message(stream: c_int, fd: c_int, flags: c_int) -> Self
    where
        Self: Sized;

    /// Returns the file stream identifier.
    fn write_open_stream(&self) -> c_int;

    /// Returns the file descriptor requested for the stream.
    fn write_open_fd(&self) -> c_int;

    /// Returns the flags used to open the stream path.
    fn write_open_flags(&self) -> c_int;
}

/// A wire message reporting completion or readiness for a file stream.
pub trait CompletionMessage {
    /// Builds a completion header.
    fn from_completion_message(stream: c_int, error: c_int) -> Self
    where
        Self: Sized;

    /// Returns the file stream identifier.
    fn completion_stream(&self) -> c_int;

    /// Returns the error reported for the stream.
    fn completion_error(&self) -> c_int;
}

impl CommandMessage for crate::types::msg_command {
    fn from_command_message(argument_count: c_int) -> Self {
        Self {
            argc: argument_count,
        }
    }
    fn command_argument_count(&self) -> c_int {
        self.argc
    }
}

macro_rules! impl_stream_message {
    ($($type:ty),+ $(,)?) => {$ (
        impl StreamMessage for $type {
            fn from_stream_message(stream: c_int) -> Self { Self { stream } }
            fn message_stream(&self) -> c_int { self.stream }
        }
    )+ };
}

impl_stream_message!(
    crate::types::msg_read_data,
    crate::types::msg_read_cancel,
    crate::types::msg_write_data,
    crate::types::msg_write_close,
);

impl ReadOpenMessage for crate::types::msg_read_open {
    fn from_read_open_message(stream: c_int, fd: c_int) -> Self {
        Self { stream, fd }
    }
    fn read_open_stream(&self) -> c_int {
        self.stream
    }
    fn read_open_fd(&self) -> c_int {
        self.fd
    }
}

impl WriteOpenMessage for crate::types::msg_write_open {
    fn from_write_open_message(stream: c_int, fd: c_int, flags: c_int) -> Self {
        Self { stream, fd, flags }
    }
    fn write_open_stream(&self) -> c_int {
        self.stream
    }
    fn write_open_fd(&self) -> c_int {
        self.fd
    }
    fn write_open_flags(&self) -> c_int {
        self.flags
    }
}

macro_rules! impl_completion_message {
    ($($type:ty),+ $(,)?) => {$ (
        impl CompletionMessage for $type {
            fn from_completion_message(stream: c_int, error: c_int) -> Self {
                Self { stream, error }
            }
            fn completion_stream(&self) -> c_int { self.stream }
            fn completion_error(&self) -> c_int { self.error }
        }
    )+ };
}

impl_completion_message!(crate::types::msg_read_done, crate::types::msg_write_ready);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{msg_command, msg_read_done, msg_read_open, msg_write_data, msg_write_open};

    #[test]
    fn protocol_headers_round_trip() {
        assert_eq!(
            msg_command::from_command_message(3).command_argument_count(),
            3
        );
        let read = msg_read_open::from_read_open_message(4, 5);
        assert_eq!((read.read_open_stream(), read.read_open_fd()), (4, 5));
        let write = msg_write_open::from_write_open_message(6, 7, 8);
        assert_eq!(
            (
                write.write_open_stream(),
                write.write_open_fd(),
                write.write_open_flags()
            ),
            (6, 7, 8)
        );
        assert_eq!(msg_write_data::from_stream_message(9).message_stream(), 9);
        let done = msg_read_done::from_completion_message(10, 11);
        assert_eq!(
            (done.completion_stream(), done.completion_error()),
            (10, 11)
        );
    }
}

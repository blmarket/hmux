//! Stable access to a file transfer's portable state.

use std::ffi::{CStr, CString};

/// The stream, path, descriptor, and completion state of a client file.
pub trait ClientFileState {
    /// Builds a client file from its portable state.
    fn from_client_file_state(
        stream: i32,
        path: Option<CString>,
        file_descriptor: i32,
        error: i32,
        closed: i32,
    ) -> Self
    where
        Self: Sized;

    /// Returns the protocol stream identifier.
    fn client_file_stream(&self) -> i32;

    /// Sets the protocol stream identifier.
    fn set_client_file_stream(&mut self, stream: i32);

    /// Returns the file path, if one has been assigned.
    fn client_file_path(&self) -> Option<&CStr>;

    /// Replaces the file path.
    fn set_client_file_path(&mut self, path: Option<CString>);

    /// Takes the file path.
    fn take_client_file_path(&mut self) -> Option<CString>;

    /// Returns the number of buffered bytes.
    fn client_file_buffer_len(&self) -> usize;

    /// Returns a copy of the buffered bytes.
    fn client_file_buffer_bytes(&mut self) -> Vec<u8>;

    /// Appends bytes to the transfer buffer.
    fn append_client_file_buffer(&mut self, data: &[u8]);

    /// Returns whether an I/O event is attached.
    fn client_file_has_event(&self) -> bool;

    /// Returns the owned file descriptor.
    fn client_file_descriptor(&self) -> i32;

    /// Sets the owned file descriptor.
    fn set_client_file_descriptor(&mut self, file_descriptor: i32);

    /// Returns the operation error number.
    fn client_file_error(&self) -> i32;

    /// Sets the operation error number.
    fn set_client_file_error(&mut self, error: i32);

    /// Returns the protocol closure state.
    fn client_file_closed(&self) -> i32;

    /// Sets the protocol closure state.
    fn set_client_file_closed(&mut self, closed: i32);
}

impl ClientFileState for crate::types::client_file {
    fn from_client_file_state(
        stream: i32,
        path: Option<CString>,
        file_descriptor: i32,
        error: i32,
        closed: i32,
    ) -> Self {
        Self {
            c: None,
            peer: None,
            tree: Default::default(),
            stream,
            path,
            buffer: Box::new(crate::reactor::ByteBuffer::new()),
            event: crate::reactor::Stream::NONE,
            fd: file_descriptor,
            error,
            closed,
            done: 0,
            cb: None,
            data: Default::default(),
        }
    }

    fn client_file_stream(&self) -> i32 {
        self.stream
    }

    fn set_client_file_stream(&mut self, stream: i32) {
        self.stream = stream;
    }

    fn client_file_path(&self) -> Option<&CStr> {
        self.path.as_deref()
    }

    fn set_client_file_path(&mut self, path: Option<CString>) {
        self.path = path;
    }

    fn take_client_file_path(&mut self) -> Option<CString> {
        self.path.take()
    }

    fn client_file_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    fn client_file_buffer_bytes(&mut self) -> Vec<u8> {
        self.buffer.as_slice().to_vec()
    }

    fn append_client_file_buffer(&mut self, data: &[u8]) {
        self.buffer.append(data);
    }

    fn client_file_has_event(&self) -> bool {
        !self.event.is_none()
    }

    fn client_file_descriptor(&self) -> i32 {
        self.fd
    }

    fn set_client_file_descriptor(&mut self, file_descriptor: i32) {
        self.fd = file_descriptor;
    }

    fn client_file_error(&self) -> i32 {
        self.error
    }

    fn set_client_file_error(&mut self, error: i32) {
        self.error = error;
    }

    fn client_file_closed(&self) -> i32 {
        self.closed
    }

    fn set_client_file_closed(&mut self, closed: i32) {
        self.closed = closed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::client_file;

    #[test]
    fn portable_state_round_trips_and_updates() {
        let mut file = client_file::from_client_file_state(7, Some(c"first".to_owned()), -1, 5, 0);
        assert_eq!(file.client_file_stream(), 7);
        assert_eq!(file.client_file_path(), Some(c"first"));
        assert_eq!(file.client_file_buffer_len(), 0);
        assert!(!file.client_file_has_event());
        assert_eq!(file.client_file_descriptor(), -1);
        assert_eq!(file.client_file_error(), 5);
        assert_eq!(file.client_file_closed(), 0);

        file.set_client_file_stream(11);
        file.set_client_file_path(Some(c"second".to_owned()));
        file.append_client_file_buffer(b"payload");
        file.set_client_file_descriptor(-7);
        file.set_client_file_error(9);
        file.set_client_file_closed(1);
        assert_eq!(file.client_file_stream(), 11);
        assert_eq!(file.client_file_path(), Some(c"second"));
        assert_eq!(file.client_file_buffer_len(), 7);
        assert_eq!(file.client_file_buffer_bytes(), b"payload");
        assert_eq!(file.client_file_descriptor(), -7);
        assert_eq!(file.client_file_error(), 9);
        assert_eq!(file.client_file_closed(), 1);
        assert_eq!(file.take_client_file_path().as_deref(), Some(c"second"));
        assert_eq!(file.client_file_path(), None);
        file.set_client_file_descriptor(-1);
    }
}

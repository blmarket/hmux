//! Stable access to imsg transport settings.

/// The process, size, descriptor, and behavior settings for an imsg channel.
pub trait ImsgBufferState {
    /// Builds transport state without an attached message queue.
    fn from_imsg_buffer(process_id: i32, max_size: u32, file_descriptor: i32, flags: i32) -> Self
    where
        Self: Sized;

    /// Returns the process identifier placed in outgoing messages.
    fn imsg_buffer_process_id(&self) -> i32;

    /// Replaces the process identifier placed in outgoing messages.
    fn set_imsg_buffer_process_id(&mut self, process_id: i32);

    /// Returns the maximum encoded message size.
    fn imsg_buffer_max_size(&self) -> u32;

    /// Replaces the maximum encoded message size.
    fn set_imsg_buffer_max_size(&mut self, max_size: u32);

    /// Returns the channel's file descriptor.
    fn imsg_buffer_file_descriptor(&self) -> i32;

    /// Replaces the channel's file descriptor.
    fn set_imsg_buffer_file_descriptor(&mut self, file_descriptor: i32);

    /// Returns the transport behavior flags.
    fn imsg_buffer_flags(&self) -> i32;

    /// Replaces the transport behavior flags.
    fn set_imsg_buffer_flags(&mut self, flags: i32);
}

impl ImsgBufferState for crate::types::imsgbuf {
    fn from_imsg_buffer(process_id: i32, max_size: u32, file_descriptor: i32, flags: i32) -> Self {
        Self {
            w: None,
            pid: process_id,
            maxsize: max_size,
            fd: file_descriptor,
            flags,
        }
    }
    fn imsg_buffer_process_id(&self) -> i32 {
        self.pid
    }
    fn set_imsg_buffer_process_id(&mut self, process_id: i32) {
        self.pid = process_id;
    }
    fn imsg_buffer_max_size(&self) -> u32 {
        self.maxsize
    }
    fn set_imsg_buffer_max_size(&mut self, max_size: u32) {
        self.maxsize = max_size;
    }
    fn imsg_buffer_file_descriptor(&self) -> i32 {
        self.fd
    }
    fn set_imsg_buffer_file_descriptor(&mut self, file_descriptor: i32) {
        self.fd = file_descriptor;
    }
    fn imsg_buffer_flags(&self) -> i32 {
        self.flags
    }
    fn set_imsg_buffer_flags(&mut self, flags: i32) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::imsgbuf;

    #[test]
    fn transport_settings_round_trip_and_change() {
        let mut state = imsgbuf::from_imsg_buffer(11, 16_384, 7, 1);
        assert_eq!(state.imsg_buffer_process_id(), 11);
        assert_eq!(state.imsg_buffer_max_size(), 16_384);
        assert_eq!(state.imsg_buffer_file_descriptor(), 7);
        assert_eq!(state.imsg_buffer_flags(), 1);
        state.set_imsg_buffer_process_id(12);
        state.set_imsg_buffer_max_size(32_768);
        state.set_imsg_buffer_file_descriptor(8);
        state.set_imsg_buffer_flags(3);
        assert_eq!(state.imsg_buffer_process_id(), 12);
        assert_eq!(state.imsg_buffer_max_size(), 32_768);
        assert_eq!(state.imsg_buffer_file_descriptor(), 8);
        assert_eq!(state.imsg_buffer_flags(), 3);
    }
}

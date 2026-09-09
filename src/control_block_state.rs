//! Stable value state for one control-mode output block.

use core::ffi::CStr;

/// Size, deferred line, and enqueue timestamp of a control output block.
pub trait ControlBlockState {
    /// Builds a control block from its complete portable values.
    fn from_control_block(size: usize, line: Option<&CStr>, timestamp: u64) -> Self
    where
        Self: Sized;

    /// Returns the number of pane-output bytes still represented by the block.
    fn control_block_size(&self) -> usize;

    /// Replaces the remaining pane-output size.
    fn set_control_block_size(&mut self, size: usize);

    /// Returns the deferred control line, when this is a line block.
    fn control_block_line(&self) -> Option<&CStr>;

    /// Replaces the deferred control line.
    fn set_control_block_line(&mut self, line: Option<&CStr>);

    /// Returns the timestamp at which the block was queued.
    fn control_block_timestamp(&self) -> u64;

    /// Replaces the enqueue timestamp.
    fn set_control_block_timestamp(&mut self, timestamp: u64);
}

impl ControlBlockState for crate::control::control_block {
    fn from_control_block(size: usize, line: Option<&CStr>, timestamp: u64) -> Self {
        Self {
            id: 0,
            size,
            line: line.map(CStr::to_owned),
            t: timestamp,
        }
    }

    fn control_block_size(&self) -> usize {
        self.size
    }

    fn set_control_block_size(&mut self, size: usize) {
        self.size = size;
    }

    fn control_block_line(&self) -> Option<&CStr> {
        self.line.as_deref()
    }

    fn set_control_block_line(&mut self, line: Option<&CStr>) {
        self.line = line.map(CStr::to_owned);
    }

    fn control_block_timestamp(&self) -> u64 {
        self.t
    }

    fn set_control_block_timestamp(&mut self, timestamp: u64) {
        self.t = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::control_block;

    #[test]
    fn block_values_round_trip_and_update() {
        let mut block = control_block::from_control_block(17, Some(c"%begin"), 23);
        assert_eq!(block.control_block_size(), 17);
        assert_eq!(block.control_block_line(), Some(c"%begin"));
        assert_eq!(block.control_block_timestamp(), 23);
        block.set_control_block_size(5);
        block.set_control_block_line(None);
        block.set_control_block_timestamp(29);
        assert_eq!(block.control_block_size(), 5);
        assert_eq!(block.control_block_line(), None);
        assert_eq!(block.control_block_timestamp(), 29);
    }
}

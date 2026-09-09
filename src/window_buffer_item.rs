//! Stable access to one buffer-mode row.

use core::ffi::CStr;
use std::ffi::CString;

/// The identity and ordering metadata for one buffer-mode row.
pub trait WindowBufferItem {
    /// Builds a row from an owned paste-buffer name and its metadata.
    fn from_window_buffer_item(name: CString, order: u32, size: usize) -> Self
    where
        Self: Sized;

    /// Returns the paste-buffer name represented by this row.
    fn window_buffer_name(&self) -> &CStr;

    /// Returns the paste buffer's creation order.
    fn window_buffer_order(&self) -> u32;

    /// Returns the paste buffer's size in bytes.
    fn window_buffer_size(&self) -> usize;
}

impl WindowBufferItem for crate::types::window_buffer_itemdata {
    fn from_window_buffer_item(name: CString, order: u32, size: usize) -> Self {
        Self {
            name: Some(name),
            order,
            size,
        }
    }

    fn window_buffer_name(&self) -> &CStr {
        self.name.as_deref().expect("a buffer row carries a name")
    }

    fn window_buffer_order(&self) -> u32 {
        self.order
    }
    fn window_buffer_size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::window_buffer_itemdata;

    #[test]
    fn name_order_and_size_round_trip() {
        let item = window_buffer_itemdata::from_window_buffer_item(c"buffer1".to_owned(), 17, 42);
        assert_eq!(item.window_buffer_name(), c"buffer1");
        assert_eq!(item.window_buffer_order(), 17);
        assert_eq!(item.window_buffer_size(), 42);
    }
}

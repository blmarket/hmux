//! Stable access to one decoded or partially decoded UTF-8 character.

/// The byte storage and decode progress carried by tmux's UTF-8 data record.
pub trait Utf8Data {
    /// Builds UTF-8 data from its complete portable state.
    fn from_utf8_data(bytes: [u8; 32], have: u8, size: u8, width: u8) -> Self;

    /// Returns the complete byte storage.
    fn utf8_bytes(&self) -> &[u8; 32];

    /// Returns the complete mutable byte storage.
    fn utf8_bytes_mut(&mut self) -> &mut [u8; 32];

    /// Returns how many bytes have arrived.
    fn utf8_have(&self) -> u8;

    /// Sets how many bytes have arrived.
    fn set_utf8_have(&mut self, have: u8);

    /// Returns the encoded character size.
    fn utf8_size(&self) -> u8;

    /// Sets the encoded character size.
    fn set_utf8_size(&mut self, size: u8);

    /// Returns the character's display width.
    fn utf8_width(&self) -> u8;

    /// Sets the character's display width.
    fn set_utf8_width(&mut self, width: u8);

    /// Replaces the decode progress and display width together.
    fn set_utf8_metadata(&mut self, have: u8, size: u8, width: u8) {
        self.set_utf8_have(have);
        self.set_utf8_size(size);
        self.set_utf8_width(width);
    }
}

impl Utf8Data for crate::text::utf8_data {
    fn from_utf8_data(bytes: [u8; 32], have: u8, size: u8, width: u8) -> Self {
        Self {
            data: bytes,
            have,
            size,
            width,
        }
    }

    fn utf8_bytes(&self) -> &[u8; 32] {
        &self.data
    }

    fn utf8_bytes_mut(&mut self) -> &mut [u8; 32] {
        &mut self.data
    }

    fn utf8_have(&self) -> u8 {
        self.have
    }

    fn set_utf8_have(&mut self, have: u8) {
        self.have = have;
    }

    fn utf8_size(&self) -> u8 {
        self.size
    }

    fn set_utf8_size(&mut self, size: u8) {
        self.size = size;
    }

    fn utf8_width(&self) -> u8 {
        self.width
    }

    fn set_utf8_width(&mut self, width: u8) {
        self.width = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::utf8_data;

    #[test]
    fn utf8_data_round_trips_and_updates() {
        let mut bytes = [0; 32];
        bytes[..3].copy_from_slice("界".as_bytes());
        let mut data = utf8_data::from_utf8_data(bytes, 2, 3, 2);
        assert_eq!(&data.utf8_bytes()[..3], "界".as_bytes());
        assert_eq!(
            (data.utf8_have(), data.utf8_size(), data.utf8_width()),
            (2, 3, 2)
        );

        data.utf8_bytes_mut()[0] = b'x';
        data.set_utf8_metadata(1, 1, 1);
        assert_eq!(data.utf8_bytes()[0], b'x');
        assert_eq!(
            (data.utf8_have(), data.utf8_size(), data.utf8_width()),
            (1, 1, 1)
        );
    }
}

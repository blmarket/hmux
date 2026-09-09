//! Stable access to systemd 128-bit identifiers.

/// A systemd 128-bit identifier with byte and machine-word views.
pub trait SystemdId128 {
    /// Builds an identifier from its byte representation.
    fn from_systemd_id128_bytes(bytes: [u8; 16]) -> Self
    where
        Self: Sized;

    /// Returns the identifier as bytes.
    fn systemd_id128_bytes(&self) -> [u8; 16];

    /// Replaces the identifier through its byte view.
    fn set_systemd_id128_bytes(&mut self, bytes: [u8; 16]);

    /// Returns the identifier as two native-endian 64-bit words.
    fn systemd_id128_qwords(&self) -> [u64; 2];

    /// Replaces the identifier through its native-endian word view.
    fn set_systemd_id128_qwords(&mut self, qwords: [u64; 2]);
}

impl SystemdId128 for crate::compat::sd_id128 {
    fn from_systemd_id128_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }
    fn systemd_id128_bytes(&self) -> [u8; 16] {
        unsafe { self.bytes }
    }
    fn set_systemd_id128_bytes(&mut self, bytes: [u8; 16]) {
        self.bytes = bytes;
    }
    fn systemd_id128_qwords(&self) -> [u64; 2] {
        unsafe { self.qwords }
    }
    fn set_systemd_id128_qwords(&mut self, qwords: [u64; 2]) {
        self.qwords = qwords;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::sd_id128;

    #[test]
    fn byte_and_word_views_share_storage() {
        let mut id = sd_id128::from_systemd_id128_bytes([0; 16]);
        id.set_systemd_id128_qwords([0x0123456789abcdef, 0xfedcba9876543210]);

        let words = id.systemd_id128_qwords();
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&words[0].to_ne_bytes());
        bytes[8..].copy_from_slice(&words[1].to_ne_bytes());
        assert_eq!(id.systemd_id128_bytes(), bytes);

        let replacement = core::array::from_fn(|index| index as u8);
        id.set_systemd_id128_bytes(replacement);
        assert_eq!(id.systemd_id128_bytes(), replacement);
    }
}

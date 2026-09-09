//! Stable access to an imsg wire header.

/// The fixed-width header at the start of an imsg message.
pub trait ImsgHeader {
    /// Builds a header from its wire fields.
    fn from_imsg_header(message_type: u32, length: u32, peer_id: u32, process_id: u32) -> Self
    where
        Self: Sized;

    /// Returns the message type.
    fn imsg_message_type(&self) -> u32;

    /// Replaces the message type.
    fn set_imsg_message_type(&mut self, message_type: u32);

    /// Returns the total encoded message length.
    fn imsg_length(&self) -> u32;

    /// Replaces the total encoded message length.
    fn set_imsg_length(&mut self, length: u32);

    /// Returns the peer identifier.
    fn imsg_peer_id(&self) -> u32;

    /// Replaces the peer identifier.
    fn set_imsg_peer_id(&mut self, peer_id: u32);

    /// Returns the process identifier carried on the wire.
    fn imsg_process_id(&self) -> u32;
}

impl ImsgHeader for crate::types::imsg_hdr {
    fn from_imsg_header(message_type: u32, length: u32, peer_id: u32, process_id: u32) -> Self {
        Self {
            type_0: message_type,
            len: length,
            peerid: peer_id,
            pid: process_id,
        }
    }
    fn imsg_message_type(&self) -> u32 {
        self.type_0
    }
    fn set_imsg_message_type(&mut self, message_type: u32) {
        self.type_0 = message_type;
    }
    fn imsg_length(&self) -> u32 {
        self.len
    }
    fn set_imsg_length(&mut self, length: u32) {
        self.len = length;
    }
    fn imsg_peer_id(&self) -> u32 {
        self.peerid
    }
    fn set_imsg_peer_id(&mut self, peer_id: u32) {
        self.peerid = peer_id;
    }
    fn imsg_process_id(&self) -> u32 {
        self.pid
    }
}

impl crate::types::imsg_hdr {
    /// Encodes the four header words in the native byte order used by imsg.
    pub(crate) fn to_ne_bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (word, value) in bytes.as_chunks_mut::<4>().0.iter_mut().zip([
            self.type_0,
            self.len,
            self.peerid,
            self.pid,
        ]) {
            word.copy_from_slice(&value.to_ne_bytes());
        }
        bytes
    }

    /// Decodes a complete imsg header without alignment requirements.
    pub(crate) fn from_ne_bytes(bytes: [u8; 16]) -> Self {
        Self {
            type_0: u32::from_ne_bytes(bytes[0..4].try_into().unwrap()),
            len: u32::from_ne_bytes(bytes[4..8].try_into().unwrap()),
            peerid: u32::from_ne_bytes(bytes[8..12].try_into().unwrap()),
            pid: u32::from_ne_bytes(bytes[12..16].try_into().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::imsg_hdr;

    #[test]
    fn wire_fields_round_trip_and_mutate() {
        let mut header = imsg_hdr::from_imsg_header(1, 16, 2, 3);
        assert_eq!(header.imsg_message_type(), 1);
        assert_eq!(header.imsg_length(), 16);
        assert_eq!(header.imsg_peer_id(), 2);
        assert_eq!(header.imsg_process_id(), 3);

        header.set_imsg_message_type(4);
        header.set_imsg_length(32);
        header.set_imsg_peer_id(5);
        assert_eq!(header.imsg_message_type(), 4);
        assert_eq!(header.imsg_length(), 32);
        assert_eq!(header.imsg_peer_id(), 5);
    }
}

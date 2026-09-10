//! Stable access to complete imsg messages.

use crate::compat::{IMSG_HEADER_SIZE, ibuf_open};

/// An imsg wire header paired with its owned body.
///
/// ```compile_fail
/// use tmux_c2rs::{ImsgMessage, types::imsg};
/// let mut message = imsg::from_imsg_message(1, 17, 0, 0, b"a");
/// let shared = message.imsg_message_data();
/// message.imsg_message_data_mut()[0] = b'b';
/// assert_eq!(shared, b"a");
/// ```
pub trait ImsgMessage {
    /// Builds a message owning a copy of its body and the supplied wire header.
    fn from_imsg_message(
        message_type: u32,
        length: u32,
        peer_id: u32,
        process_id: u32,
        data: &[u8],
    ) -> Self
    where
        Self: Sized;

    /// Returns the message type.
    fn imsg_message_type(&self) -> u32;

    /// Returns the total encoded length.
    fn imsg_message_length(&self) -> u32;

    /// Returns the peer identifier.
    fn imsg_message_peer_id(&self) -> u32;

    /// Returns the process identifier.
    fn imsg_message_process_id(&self) -> u32;

    /// Borrows the original message body, independently of the reader cursor.
    fn imsg_message_data(&self) -> &[u8];

    /// Exclusively borrows the original message body.
    fn imsg_message_data_mut(&mut self) -> &mut [u8];
}

impl ImsgMessage for crate::types::imsg {
    fn from_imsg_message(
        message_type: u32,
        length: u32,
        peer_id: u32,
        process_id: u32,
        data: &[u8],
    ) -> Self {
        let hdr = crate::types::imsg_hdr {
            type_0: message_type,
            len: length,
            peerid: peer_id,
            pid: process_id,
        };
        let end = IMSG_HEADER_SIZE
            .checked_add(data.len())
            .expect("message size fits in memory");
        let mut buf = ibuf_open(end).expect("message buffer allocated");
        buf.buf[..IMSG_HEADER_SIZE].copy_from_slice(&hdr.to_ne_bytes());
        buf.buf[IMSG_HEADER_SIZE..end].copy_from_slice(data);
        buf.rpos = IMSG_HEADER_SIZE;
        buf.wpos = end;
        Self {
            hdr,
            data: IMSG_HEADER_SIZE..end,
            buf: Some(buf),
        }
    }
    fn imsg_message_type(&self) -> u32 {
        self.hdr.type_0
    }
    fn imsg_message_length(&self) -> u32 {
        self.hdr.len
    }
    fn imsg_message_peer_id(&self) -> u32 {
        self.hdr.peerid
    }
    fn imsg_message_process_id(&self) -> u32 {
        self.hdr.pid
    }
    fn imsg_message_data(&self) -> &[u8] {
        self.buf
            .as_deref()
            .map_or(&[], |buf| &buf.buf[self.data.clone()])
    }
    fn imsg_message_data_mut(&mut self) -> &mut [u8] {
        self.buf
            .as_deref_mut()
            .map_or(&mut [], |buf| &mut buf.buf[self.data.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::imsg;

    #[test]
    fn body_borrow_keeps_the_original_range_after_reads_advance() {
        let mut message = imsg::from_imsg_message(1, 19, 2, 3, b"abc");
        let mut consumed = [0; 2];
        assert_eq!(crate::compat::imsg_get_buf(&mut message, &mut consumed), 0);
        assert_eq!(&consumed, b"ab");
        assert_eq!(message.imsg_message_data(), b"abc");
        assert_eq!({ crate::compat::imsg_get_len(&message) }, 1);
        message.imsg_message_data_mut()[2] = b'd';
        let mut last = [0; 1];
        assert_eq!(crate::compat::imsg_get_buf(&mut message, &mut last), 0);
        assert_eq!(&last, b"d");
        assert_eq!(message.imsg_message_data(), b"abd");
    }

    #[test]
    fn header_and_body_round_trip() {
        let byte = [7_u8];
        let message = imsg::from_imsg_message(1, 17, 2, 3, &byte);
        assert_eq!(message.imsg_message_type(), 1);
        assert_eq!(message.imsg_message_length(), 17);
        assert_eq!(message.imsg_message_peer_id(), 2);
        assert_eq!(message.imsg_message_process_id(), 3);
        assert_eq!(message.imsg_message_data(), &byte);
    }
}

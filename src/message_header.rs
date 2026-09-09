//! Stable access to socket message and control-message headers.

use crate::types::{socklen_t, ssize_t};
use core::ffi::c_int;
use std::io::{IoSlice, IoSliceMut};

/// Exclusive borrows of the buffers used by a socket message.
///
/// ```compile_fail
/// use std::io::IoSliceMut;
/// use tmux_c2rs::{MessageHeader, SocketMessage};
/// fn escaped() -> SocketMessage<'static, 'static> {
///     let mut bytes = [0; 8];
///     let mut vectors = [IoSliceMut::new(&mut bytes)];
///     SocketMessage::from_message_header(None, &mut vectors, &mut [], 0)
/// }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::{MessageHeader, SocketMessage};
/// let mut storage = [0; 8];
/// let mut message = SocketMessage::from_message_header(None, &mut [], &mut storage, 0);
/// let before = message.message_control();
/// message.message_control_mut().fill(1);
/// assert_eq!(before[0], 0);
/// ```
pub trait MessageHeader<'a, 'data: 'a> {
    /// Retains borrows of the optional address, I/O vectors, and control storage.
    fn from_message_header(
        name: Option<&'a mut [u8]>,
        io_vectors: &'a mut [IoSliceMut<'data>],
        control: &'a mut [u8],
        flags: c_int,
    ) -> Self
    where
        Self: Sized;

    /// Borrows the available peer-address bytes, bounded by their storage.
    fn message_name(&self) -> Option<&[u8]>;

    /// Exclusively borrows the available peer-address bytes.
    fn message_name_mut(&mut self) -> Option<&mut [u8]>;

    /// Returns the address length, which may exceed storage after a truncated receive.
    fn message_name_length(&self) -> socklen_t;

    /// Borrows the scatter/gather vector and its initialized regions.
    fn message_io_vectors(&self) -> &[IoSliceMut<'data>];

    /// Exclusively borrows the scatter/gather vector and its regions.
    fn message_io_vectors_mut(&mut self) -> &mut [IoSliceMut<'data>];

    /// Returns the number of scatter/gather entries.
    fn message_io_vector_count(&self) -> usize;

    /// Borrows the available ancillary bytes.
    fn message_control(&self) -> &[u8];

    /// Exclusively borrows the available ancillary bytes.
    fn message_control_mut(&mut self) -> &mut [u8];

    /// Returns the ancillary-data buffer length.
    fn message_control_length(&self) -> usize;

    /// Returns the message flags.
    fn message_flags(&self) -> c_int;

    /// Replaces the scatter/gather vector with another exclusive borrow.
    fn set_message_io_vectors(&mut self, io_vectors: &'a mut [IoSliceMut<'data>]);

    /// Replaces the ancillary-data buffer with another exclusive borrow.
    fn set_message_control(&mut self, control: &'a mut [u8]);

    /// Replaces the ancillary-data length if it fits the retained storage.
    fn set_message_control_length(&mut self, length: usize) -> bool;
}

/// Socket receive storage retained for the lifetime of the message.
pub struct SocketMessage<'a, 'data: 'a> {
    name: Option<&'a mut [u8]>,
    name_length: socklen_t,
    io_vectors: &'a mut [IoSliceMut<'data>],
    control: &'a mut [u8],
    control_length: usize,
    flags: c_int,
}

impl<'a, 'data: 'a> MessageHeader<'a, 'data> for SocketMessage<'a, 'data> {
    fn from_message_header(
        name: Option<&'a mut [u8]>,
        io_vectors: &'a mut [IoSliceMut<'data>],
        control: &'a mut [u8],
        flags: c_int,
    ) -> Self {
        Self {
            name_length: name.as_ref().map_or(0, |name| {
                name.len().try_into().expect("a socket address length")
            }),
            name,
            io_vectors,
            control_length: control.len(),
            control,
            flags,
        }
    }
    fn message_name(&self) -> Option<&[u8]> {
        self.name
            .as_deref()
            .map(|name| &name[..name.len().min(self.name_length as usize)])
    }
    fn message_name_mut(&mut self) -> Option<&mut [u8]> {
        self.name.as_deref_mut().map(|name| {
            let length = name.len().min(self.name_length as usize);
            &mut name[..length]
        })
    }
    fn message_name_length(&self) -> socklen_t {
        self.name_length
    }
    fn message_io_vectors(&self) -> &[IoSliceMut<'data>] {
        self.io_vectors
    }
    fn message_io_vectors_mut(&mut self) -> &mut [IoSliceMut<'data>] {
        self.io_vectors
    }
    fn message_io_vector_count(&self) -> usize {
        self.io_vectors.len()
    }
    fn message_control(&self) -> &[u8] {
        &self.control[..self.control_length]
    }
    fn message_control_mut(&mut self) -> &mut [u8] {
        &mut self.control[..self.control_length]
    }
    fn message_control_length(&self) -> usize {
        self.control_length
    }
    fn message_flags(&self) -> c_int {
        self.flags
    }
    fn set_message_io_vectors(&mut self, io_vectors: &'a mut [IoSliceMut<'data>]) {
        self.io_vectors = io_vectors;
    }
    fn set_message_control(&mut self, control: &'a mut [u8]) {
        self.control_length = control.len();
        self.control = control;
    }
    fn set_message_control_length(&mut self, length: usize) -> bool {
        if length > self.control.len() {
            return false;
        }
        self.control_length = length;
        true
    }
}

impl SocketMessage<'_, '_> {
    pub(crate) fn receive(&mut self, fd: c_int, flags: c_int) -> ssize_t {
        let mut address = [0; core::mem::size_of::<libc::sockaddr_storage>()];
        let mut header = libc::msghdr {
            msg_name: if self.name.is_some() {
                address.as_mut_ptr().cast()
            } else {
                core::ptr::null_mut()
            },
            msg_namelen: if self.name.is_some() {
                address.len() as socklen_t
            } else {
                0
            },
            msg_iov: self.io_vectors.as_mut_ptr().cast(),
            msg_iovlen: self.io_vectors.len(),
            msg_control: if self.control_length == 0 {
                core::ptr::null_mut()
            } else {
                self.control.as_mut_ptr().cast()
            },
            msg_controllen: self.control_length,
            msg_flags: self.flags,
        };
        let result = unsafe { libc::recvmsg(fd, &raw mut header, flags) as ssize_t };
        if result >= 0 {
            if let Some(name) = self.name.as_deref_mut() {
                let copied = name
                    .len()
                    .min(self.name_length as usize)
                    .min(header.msg_namelen as usize)
                    .min(address.len());
                name[..copied].copy_from_slice(&address[..copied]);
            }
            self.name_length = header.msg_namelen;
        }
        self.control_length = header.msg_controllen.min(self.control.len());
        self.flags = header.msg_flags;
        result
    }
}

pub(crate) fn send_socket_message(
    fd: c_int,
    vectors: &[IoSlice<'_>],
    control: &[u8],
    flags: c_int,
) -> ssize_t {
    let header = libc::msghdr {
        msg_name: core::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: vectors.as_ptr().cast_mut().cast(),
        msg_iovlen: vectors.len(),
        msg_control: if control.is_empty() {
            core::ptr::null_mut()
        } else {
            control.as_ptr().cast_mut().cast()
        },
        msg_controllen: control.len(),
        msg_flags: 0,
    };
    unsafe { libc::sendmsg(fd, &raw const header, flags) as ssize_t }
}

/// One ancillary-data header and its payload, borrowed from initialized storage.
///
/// ```compile_fail
/// use tmux_c2rs::{ControlMessage, ControlMessageHeader};
/// fn escaped() -> ControlMessage<'static> {
///     let mut storage = [0; 24];
///     ControlMessage::from_control_message_header(&mut storage, 20, 1, 1).unwrap()
/// }
/// ```
pub trait ControlMessageHeader<'a> {
    /// Initializes a header in the supplied storage, rejecting invalid lengths.
    fn from_control_message_header(
        storage: &'a mut [u8],
        length: usize,
        level: c_int,
        kind: c_int,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Returns the total header and payload length.
    fn control_message_length(&self) -> usize;

    /// Returns the originating protocol level.
    fn control_message_level(&self) -> c_int;

    /// Returns the protocol-specific message kind.
    fn control_message_kind(&self) -> c_int;

    /// Borrows the initialized payload within the declared message length.
    fn control_message_data(&self) -> &[u8];

    /// Exclusively borrows the initialized payload.
    fn control_message_data_mut(&mut self) -> &mut [u8];

    /// Replaces the length if it includes the header and fits the storage.
    fn set_control_message_length(&mut self, length: usize) -> bool;

    /// Replaces the originating protocol level.
    fn set_control_message_level(&mut self, level: c_int);

    /// Replaces the protocol-specific message kind.
    fn set_control_message_kind(&mut self, kind: c_int);
}

/// The platform ancillary header size, including padding before its payload.
pub const CONTROL_MESSAGE_HEADER_SIZE: usize = size_of::<libc::cmsghdr>();

/// A bounded view of one ancillary message and its trailing alignment padding.
pub struct ControlMessage<'a> {
    storage: &'a mut [u8],
}

impl<'a> ControlMessageHeader<'a> for ControlMessage<'a> {
    fn from_control_message_header(
        storage: &'a mut [u8],
        length: usize,
        level: c_int,
        kind: c_int,
    ) -> Option<Self> {
        if !(CONTROL_MESSAGE_HEADER_SIZE..=storage.len()).contains(&length) {
            return None;
        }
        let mut message = ControlMessage { storage };
        message.set_control_message_length(length);
        message.set_control_message_level(level);
        message.set_control_message_kind(kind);
        Some(message)
    }

    fn control_message_length(&self) -> usize {
        usize::from_ne_bytes(self.storage[..size_of::<usize>()].try_into().unwrap())
    }

    fn control_message_level(&self) -> c_int {
        c_int::from_ne_bytes(
            self.storage[size_of::<usize>()..size_of::<usize>() + 4]
                .try_into()
                .unwrap(),
        )
    }

    fn control_message_kind(&self) -> c_int {
        c_int::from_ne_bytes(
            self.storage[size_of::<usize>() + 4..size_of::<usize>() + 8]
                .try_into()
                .unwrap(),
        )
    }

    fn control_message_data(&self) -> &[u8] {
        &self.storage[CONTROL_MESSAGE_HEADER_SIZE..self.control_message_length()]
    }

    fn control_message_data_mut(&mut self) -> &mut [u8] {
        let length = self.control_message_length();
        &mut self.storage[CONTROL_MESSAGE_HEADER_SIZE..length]
    }

    fn set_control_message_length(&mut self, length: usize) -> bool {
        if !(CONTROL_MESSAGE_HEADER_SIZE..=self.storage.len()).contains(&length) {
            return false;
        }
        self.storage[..size_of::<usize>()].copy_from_slice(&length.to_ne_bytes());
        true
    }

    fn set_control_message_level(&mut self, level: c_int) {
        self.storage[size_of::<usize>()..size_of::<usize>() + 4]
            .copy_from_slice(&level.to_ne_bytes());
    }

    fn set_control_message_kind(&mut self, kind: c_int) {
        self.storage[size_of::<usize>() + 4..size_of::<usize>() + 8]
            .copy_from_slice(&kind.to_ne_bytes());
    }
}

/// Iterates over disjoint, validated messages in an ancillary buffer.
pub struct ControlMessages<'a> {
    remaining: &'a mut [u8],
}

impl<'a> ControlMessages<'a> {
    /// Borrows initialized bytes returned by the socket operation.
    pub fn new(storage: &'a mut [u8]) -> Self {
        Self { remaining: storage }
    }
}

impl<'a> Iterator for ControlMessages<'a> {
    type Item = ControlMessage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let storage = std::mem::take(&mut self.remaining);
        if storage.len() < CONTROL_MESSAGE_HEADER_SIZE {
            return None;
        }
        let length = usize::from_ne_bytes(storage[..size_of::<usize>()].try_into().unwrap());
        if !(CONTROL_MESSAGE_HEADER_SIZE..=storage.len()).contains(&length) {
            return None;
        }
        let aligned = length.checked_add(size_of::<usize>() - 1)? & !(size_of::<usize>() - 1);
        let split = aligned.min(storage.len());
        let (message, remaining) = storage.split_at_mut(split);
        self.remaining = remaining;
        Some(ControlMessage { storage: message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiving_an_address_larger_than_its_storage_keeps_the_view_bounded() {
        use std::os::fd::AsRawFd;
        let receiver = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut expected = [0; core::mem::size_of::<libc::sockaddr_in>()];
        expected[..2].copy_from_slice(&(libc::AF_INET as u16).to_ne_bytes());
        expected[2..4].copy_from_slice(&sender.local_addr().unwrap().port().to_be_bytes());
        expected[4..8].copy_from_slice(&[127, 0, 0, 1]);
        for capacity in [0, 1, 2, 15, 16, 128, 256] {
            sender
                .send_to(b"abc", receiver.local_addr().unwrap())
                .unwrap();
            let mut data = [0; 4];
            let mut vectors = [IoSliceMut::new(&mut data)];
            let mut storage = vec![0xa5; capacity + 2];
            {
                let address = &mut storage[1..=capacity];
                let mut message =
                    SocketMessage::from_message_header(Some(address), &mut vectors, &mut [], 0);
                assert_eq!(message.receive(receiver.as_raw_fd(), 0), 3);
                assert_eq!(message.message_name_length() as usize, expected.len());
                let copied = capacity.min(expected.len());
                assert_eq!(message.message_name().unwrap(), &expected[..copied]);
                assert_eq!(message.message_name_mut().unwrap(), &expected[..copied]);
                assert_eq!(&message.message_io_vectors()[0][..3], b"abc");
                assert!(message.message_control().is_empty());
            }
            assert_eq!(storage[0], 0xa5);
            assert!(
                storage[1 + capacity.min(expected.len())..]
                    .iter()
                    .all(|&byte| byte == 0xa5)
            );
        }
    }

    #[test]
    fn control_lengths_must_fit_the_backing_storage() {
        let mut storage = [0; 24];
        assert!(ControlMessage::from_control_message_header(&mut storage, 15, 1, 2).is_none());
        assert!(ControlMessage::from_control_message_header(&mut storage, 25, 1, 2).is_none());
        assert_eq!(storage, [0; 24]);
        let mut message =
            ControlMessage::from_control_message_header(&mut storage, 20, 1, 2).unwrap();
        message
            .control_message_data_mut()
            .copy_from_slice(&7_i32.to_ne_bytes());
        assert!(!message.set_control_message_length(25));
        assert!(!message.set_control_message_length(15));
        assert_eq!(message.control_message_data(), 7_i32.to_ne_bytes());
        assert!(message.set_control_message_length(24));
        assert_eq!(&message.control_message_data()[4..], &[0; 4]);
    }

    #[test]
    fn control_iteration_borrows_disjoint_messages_and_excludes_padding() {
        let mut storage = [0; 41];
        ControlMessage::from_control_message_header(&mut storage[..24], 20, 1, 2).unwrap();
        ControlMessage::from_control_message_header(&mut storage[24..], 17, 3, 4).unwrap();
        let mut messages = ControlMessages::new(&mut storage);
        let mut first = messages.next().unwrap();
        let mut second = messages.next().unwrap();
        assert!(messages.next().is_none());
        first.control_message_data_mut().fill(5);
        second.control_message_data_mut().fill(6);
        assert_eq!(first.control_message_data(), &[5; 4]);
        assert_eq!(second.control_message_data(), &[6]);
        assert_eq!(second.control_message_level(), 3);
        assert_eq!(second.control_message_kind(), 4);
        assert_eq!(&storage[20..24], &[0; 4]);
        assert_eq!(storage[40], 6);
    }

    #[test]
    fn control_iteration_stops_at_truncated_or_invalid_headers() {
        for length in [0, 15, 25, usize::MAX] {
            let mut storage = [0; 24];
            storage[..size_of::<usize>()].copy_from_slice(&length.to_ne_bytes());
            let mut messages = ControlMessages::new(&mut storage);
            assert!(messages.next().is_none());
            assert!(messages.next().is_none());
        }
        assert!(ControlMessages::new(&mut [0; 15]).next().is_none());
    }

    #[test]
    fn socket_headers_borrow_and_update_their_regions() {
        let mut bytes = [0; 7];
        let mut vectors = [IoSliceMut::new(&mut bytes)];
        let mut name = [0; 4];
        let mut control = [0; 32];
        let mut header =
            SocketMessage::from_message_header(Some(&mut name), &mut vectors, &mut control, 4);
        assert_eq!(header.message_name_length(), 4);
        header.message_name_mut().unwrap()[1] = 3;
        header.message_io_vectors_mut()[0][2] = 5;
        assert_eq!(header.message_io_vector_count(), 1);
        assert_eq!(header.message_io_vectors()[0][2], 5);
        assert_eq!(header.message_control_length(), 32);
        assert!(!header.set_message_control_length(33));
        assert_eq!(header.message_control_length(), 32);
        assert!(header.set_message_control_length(4));
        header.message_control_mut().fill(7);
        assert_eq!(header.message_control(), &[7; 4]);
        assert_eq!(header.message_flags(), 4);
        assert_eq!(name, [0, 3, 0, 0]);
        assert_eq!(bytes, [0, 0, 5, 0, 0, 0, 0]);
        assert_eq!(&control[..4], &[7; 4]);
        assert_eq!(&control[4..], &[0; 28]);
    }
}

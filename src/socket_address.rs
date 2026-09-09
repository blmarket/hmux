//! Stable access to operating-system socket addresses.

use core::ffi::{CStr, c_char};

/// A generic socket address in its operating-system wire shape.
pub trait SocketAddress {
    /// Builds an address from its family and opaque address bytes.
    fn from_socket_address(family: u16, data: [c_char; 14]) -> Self
    where
        Self: Sized;

    /// Returns the address family.
    fn socket_address_family(&self) -> u16;

    /// Returns the opaque address bytes.
    fn socket_address_data(&self) -> [c_char; 14];
}

/// An IPv4 socket address in its operating-system wire shape.
pub trait InternetSocketAddress {
    /// Builds an IPv4 address from network-order port and address words.
    fn from_internet_socket_address(family: u16, port: u16, address: u32) -> Self
    where
        Self: Sized;

    /// Returns the address family.
    fn internet_socket_family(&self) -> u16;

    /// Returns the port in network byte order.
    fn internet_socket_port(&self) -> u16;

    /// Returns the IPv4 address in network byte order.
    fn internet_socket_address(&self) -> u32;
}

/// A Unix-domain socket address with fixed-capacity path storage.
pub trait UnixSocketAddress: Default {
    /// Replaces the family and path, returning false when the path does not fit.
    fn set_unix_socket_address(&mut self, family: u16, path: &CStr) -> bool;

    /// Returns the address family.
    fn unix_socket_family(&self) -> u16;

    /// Returns the NUL-terminated socket path.
    fn unix_socket_path(&self) -> &CStr;
}

impl SocketAddress for crate::types::sockaddr {
    fn from_socket_address(family: u16, data: [c_char; 14]) -> Self {
        Self {
            sa_family: family,
            sa_data: data,
        }
    }

    fn socket_address_family(&self) -> u16 {
        self.sa_family
    }

    fn socket_address_data(&self) -> [c_char; 14] {
        self.sa_data
    }
}

impl InternetSocketAddress for crate::types::sockaddr_in {
    fn from_internet_socket_address(family: u16, port: u16, address: u32) -> Self {
        Self {
            sin_family: family,
            sin_port: port,
            sin_addr: crate::types::in_addr { s_addr: address },
            sin_zero: [0; 8],
        }
    }

    fn internet_socket_family(&self) -> u16 {
        self.sin_family
    }

    fn internet_socket_port(&self) -> u16 {
        self.sin_port
    }

    fn internet_socket_address(&self) -> u32 {
        self.sin_addr.s_addr
    }
}

impl UnixSocketAddress for crate::types::sockaddr_un {
    fn set_unix_socket_address(&mut self, family: u16, path: &CStr) -> bool {
        let bytes = path.to_bytes_with_nul();
        if bytes.len() > self.sun_path.len() {
            return false;
        }
        self.sun_family = family;
        self.sun_path.fill(0);
        self.sun_path[..bytes.len()].copy_from_slice(bytes);
        true
    }

    fn unix_socket_family(&self) -> u16 {
        self.sun_family
    }

    fn unix_socket_path(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.sun_path)
            .expect("a Unix socket address carries a terminated path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{sockaddr, sockaddr_in, sockaddr_un};

    #[test]
    fn generic_address_round_trips() {
        let address = sockaddr::from_socket_address(7, [3; 14]);
        assert_eq!(address.socket_address_family(), 7);
        assert_eq!(address.socket_address_data(), [3; 14]);
    }

    #[test]
    fn internet_address_round_trips() {
        let address = sockaddr_in::from_internet_socket_address(2, 0x1234, 0x0102_0304);
        assert_eq!(address.internet_socket_family(), 2);
        assert_eq!(address.internet_socket_port(), 0x1234);
        assert_eq!(address.internet_socket_address(), 0x0102_0304);
    }

    #[test]
    fn unix_address_accepts_only_paths_that_fit() {
        let mut address = sockaddr_un::default();
        assert!(address.set_unix_socket_address(1, c"/tmp/hmux.sock"));
        assert_eq!(address.unix_socket_family(), 1);
        assert_eq!(address.unix_socket_path(), c"/tmp/hmux.sock");

        let longest = std::ffi::CString::new(vec![0xff; 107]).unwrap();
        assert!(address.set_unix_socket_address(1, &longest));
        assert_eq!(address.unix_socket_path(), longest.as_c_str());

        let too_long = std::ffi::CString::new(vec![b'x'; 108]).unwrap();
        assert!(!address.set_unix_socket_address(1, &too_long));
    }

    #[test]
    fn unix_address_layout_matches_the_native_socket_address() {
        assert_eq!(size_of::<sockaddr_un>(), size_of::<libc::sockaddr_un>());
        assert_eq!(align_of::<sockaddr_un>(), align_of::<libc::sockaddr_un>());
        assert_eq!(
            core::mem::offset_of!(sockaddr_un, sun_path),
            core::mem::offset_of!(libc::sockaddr_un, sun_path),
        );
    }
}

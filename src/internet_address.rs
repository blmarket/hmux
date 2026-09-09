//! Stable access to an IPv4 address in its operating-system wire shape.

/// An IPv4 address stored as one network-order word.
pub trait InternetAddress {
    /// Builds an IPv4 address from its network-order representation.
    fn from_ipv4_network_order(address: u32) -> Self
    where
        Self: Sized;

    /// Returns the address in network byte order.
    fn ipv4_network_order(&self) -> u32;
}

impl InternetAddress for crate::types::in_addr {
    fn from_ipv4_network_order(address: u32) -> Self {
        Self { s_addr: address }
    }
    fn ipv4_network_order(&self) -> u32 {
        self.s_addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::in_addr;

    #[test]
    fn network_order_word_round_trips() {
        let address = in_addr::from_ipv4_network_order(0x0102_0304);
        assert_eq!(address.ipv4_network_order(), 0x0102_0304);
    }
}

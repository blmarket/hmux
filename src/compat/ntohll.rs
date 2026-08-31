//! `ntohll`: a 64-bit value in network byte order as the same value in host
//! byte order.
//!
//! tmux's compat function splits the value into its two 32-bit halves, pushes
//! each through `ntohl` and puts the two back with their places exchanged. On
//! a little-endian host — the one this crate is transpiled for, and what the
//! transpiled body had baked in, `ntohl` being a byte swap there — swapping
//! each half and exchanging the halves is the reversal of all eight bytes,
//! which is what `u64::from_be` is. On a big-endian host `ntohl` is the
//! identity, so the C would exchange the halves and swap nothing, while
//! `u64::from_be` is the identity for the whole word; that difference is out
//! of reach of this crate, whose transpiled types and layouts are the
//! little-endian target's.
//!
//! Coverage exemptions: none.
pub use crate::types::*;

/// `v`, read from network byte order, in host byte order.
pub fn ntohll(v: uint64_t) -> uint64_t {
    uint64_t::from_be(v)
}

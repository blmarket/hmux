//! `htonll`: a 64-bit value in host byte order as the same value in network
//! byte order.
//!
//! tmux's compat function splits the value into its two 32-bit halves, pushes
//! each through `htonl` and puts the two back with their places exchanged. On
//! a little-endian host — the one this crate is transpiled for, and what the
//! transpiled body had baked in, `htonl` being a byte swap there — swapping
//! each half and exchanging the halves is the reversal of all eight bytes,
//! which is what `u64::to_be` is. On a big-endian host `htonl` is the
//! identity, so the C would exchange the halves and swap nothing, while
//! `u64::to_be` is the identity for the whole word; that difference is out of
//! reach of this crate, whose transpiled types and layouts are the
//! little-endian target's.
//!
//! Coverage exemptions: none.
pub use crate::types::*;

/// `v` in network byte order.
pub fn htonll(v: uint64_t) -> uint64_t {
    v.to_be()
}

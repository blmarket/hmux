//! Unit tests for [`crate::compat`].
//!
//! The translated `ntohll` pushes each 32-bit half of its argument through the
//! glibc byte swapper and recombines the halves the other way round, so its
//! whole observable behaviour is the reversal of the eight bytes of the
//! argument. It is pure arithmetic over its input and touches none of the
//! process-wide state the server keeps in statics, so no turn at the
//! [`crate::tests::test_fixtures::globals`] mutex is wanted here. Every
//! expectation is derived from the argument's own bytes instead of being
//! written down against one layout, so the suite reads the same on little-
//! and big-endian hosts.

use crate::compat::ntohll;
use crate::types::*;

/// The eight bytes of `v`, reversed and read back as a `uint64_t`: the answer
/// `ntohll` owes for `v`, whichever way round the host keeps its bytes.
fn reversed_bytes(v: uint64_t) -> uint64_t {
    let mut b = v.to_ne_bytes();
    b.reverse();
    uint64_t::from_ne_bytes(b)
}

#[test]
fn ntohll_answers_zero_for_zero() {
    {
        assert_eq!(ntohll(0 as uint64_t), 0 as uint64_t);
    }
}

#[test]
fn ntohll_reverses_the_eight_bytes_of_its_argument() {
    {
        assert_eq!(
            ntohll(0x0123456789abcdef as uint64_t),
            0xefcdab8967452301 as uint64_t
        );
        assert_eq!(
            ntohll(0x1122334455667788 as uint64_t),
            0x8877665544332211 as uint64_t
        );
        assert_eq!(
            ntohll(0xdeadbeefcafebabe as uint64_t),
            reversed_bytes(0xdeadbeefcafebabe)
        );
    }
}

#[test]
fn ntohll_carries_the_boundary_values_across_the_word() {
    {
        assert_eq!(ntohll(u64::MAX as uint64_t), u64::MAX as uint64_t);
        assert_eq!(ntohll(1 as uint64_t), (1 as uint64_t) << 56);
        assert_eq!(ntohll((1 as uint64_t) << 56), 1 as uint64_t);
        assert_eq!(ntohll(0xff as uint64_t), (0xff as uint64_t) << 56);
        assert_eq!(ntohll((0xff as uint64_t) << 56), 0xff as uint64_t);
        assert_eq!(
            ntohll(0xffffffff as uint64_t),
            0xffffffff00000000 as uint64_t
        );
        assert_eq!(
            ntohll(0xffffffff00000000 as uint64_t),
            0xffffffff as uint64_t
        );
        assert_eq!(ntohll(0x8000000000000000 as uint64_t), 0x80 as uint64_t);
    }
}

#[test]
fn ntohll_walks_a_lone_byte_to_every_position_and_back() {
    {
        for i in 0..8 {
            let byte_at_i = (0xffu64 << (i * 8)) as uint64_t;
            assert_eq!(ntohll(byte_at_i), (0xffu64 << ((7 - i) * 8)) as uint64_t);

            let bit_at_i = (1u64 << (i * 8)) as uint64_t;
            assert_eq!(ntohll(bit_at_i), (1u64 << ((7 - i) * 8)) as uint64_t);
            assert_eq!(ntohll(ntohll(bit_at_i)), bit_at_i);
        }
    }
}

#[test]
fn ntohll_turns_an_octet_string_into_its_mirror_under_every_interpretation() {
    let wire: [u8; 8] = [0x80, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x01];
    let mut mirror = wire;
    mirror.reverse();
    {
        assert_eq!(
            ntohll(uint64_t::from_be_bytes(wire)),
            uint64_t::from_be_bytes(mirror)
        );
        assert_eq!(
            ntohll(uint64_t::from_le_bytes(wire)),
            uint64_t::from_le_bytes(mirror)
        );
        assert_eq!(
            ntohll(uint64_t::from_ne_bytes(wire)),
            uint64_t::from_ne_bytes(mirror)
        );
    }
}

#[test]
fn ntohll_undoing_itself_restores_any_value() {
    {
        let samples: [uint64_t; 8] = [
            0,
            1,
            1u64 << 63,
            u64::MAX,
            0x00000000ffffffff,
            0x0123456789abcdef,
            0xfedcba9876543210,
            0xa5a5a5a55a5a5a5a,
        ];
        for v in samples {
            assert_eq!(ntohll(ntohll(v)), v);
        }
    }
}

#[test]
fn ntohll_matches_a_plain_byte_reversal_over_a_sweep() {
    {
        let mut x: uint64_t = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..1024 {
            assert_eq!(ntohll(x), reversed_bytes(x));
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
        }
    }
}

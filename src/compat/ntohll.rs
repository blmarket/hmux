use crate::src::shared::abi::*;
use crate::src::shared::abi::{__uint32_t, uint32_t};
#[inline]
unsafe fn __bswap_32(mut __bsx: __uint32_t) -> __uint32_t {
    return (__bsx & 0xff000000 as __uint32_t) >> 24 as ::core::ffi::c_int
        | (__bsx & 0xff0000 as __uint32_t) >> 8 as ::core::ffi::c_int
        | (__bsx & 0xff00 as __uint32_t) << 8 as ::core::ffi::c_int
        | (__bsx & 0xff as __uint32_t) << 24 as ::core::ffi::c_int;
}
pub unsafe fn ntohll(mut v: uint64_t) -> uint64_t {
    let mut b: uint32_t = 0;
    let mut t: uint32_t = 0;
    b = __bswap_32((v & 0xffffffff as uint64_t) as __uint32_t) as uint32_t;
    t = __bswap_32((v >> 32 as ::core::ffi::c_int) as __uint32_t) as uint32_t;
    return (b as uint64_t) << 32 as ::core::ffi::c_int | t as uint64_t;
}

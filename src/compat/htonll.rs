use crate::src::shared::abi::*;
pub type __uint32_t = u32;
pub type uint32_t = __uint32_t;
#[inline]
unsafe extern "C" fn __bswap_32(mut __bsx: __uint32_t) -> __uint32_t {
    return (__bsx & 0xff000000 as __uint32_t) >> 24 as ::core::ffi::c_int
        | (__bsx & 0xff0000 as __uint32_t) >> 8 as ::core::ffi::c_int
        | (__bsx & 0xff00 as __uint32_t) << 8 as ::core::ffi::c_int
        | (__bsx & 0xff as __uint32_t) << 24 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn htonll(mut v: uint64_t) -> uint64_t {
    let mut b: uint32_t = 0;
    let mut t: uint32_t = 0;
    b = __bswap_32((v & 0xffffffff as uint64_t) as __uint32_t) as uint32_t;
    t = __bswap_32((v >> 32 as ::core::ffi::c_int) as __uint32_t) as uint32_t;
    return (b as uint64_t) << 32 as ::core::ffi::c_int | t as uint64_t;
}

#[cfg(test)]
mod tests {
    use super::htonll;

    #[test]
    fn converts_to_network_byte_order() {
        assert_eq!(unsafe { htonll(0x0123_4567_89ab_cdef) }, 0xefcd_ab89_6745_2301);
    }
}

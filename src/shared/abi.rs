//! Platform ABI declarations shared by all translated modules.
//!
//! These aliases and `timeval` were emitted identically by every C2Rust
//! translation unit.  Keeping one definition prevents otherwise identical C
//! function signatures from acquiring distinct Rust type identities.

pub type __u_char = ::core::ffi::c_uchar;
pub type __u_short = ::core::ffi::c_ushort;
pub type __u_int = ::core::ffi::c_uint;
pub type __uint8_t = u8;
pub type __uint64_t = u64;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;

pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type pid_t = __pid_t;
pub type time_t = __time_t;
pub type size_t = usize;
pub type uint8_t = __uint8_t;
pub type uint64_t = __uint64_t;
pub type bitstr_t = ::core::ffi::c_uchar;

pub type cc_t = ::core::ffi::c_uchar;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}

#[cfg(test)]
mod tests {
    use super::timeval;
    use ::core::mem::{align_of, offset_of, size_of};

    #[test]
    fn timeval_layout_matches_linux_amd64_baseline() {
        assert_eq!(size_of::<timeval>(), 16);
        assert_eq!(align_of::<timeval>(), 8);
        assert_eq!(offset_of!(timeval, tv_sec), 0);
        assert_eq!(offset_of!(timeval, tv_usec), 8);
    }
}

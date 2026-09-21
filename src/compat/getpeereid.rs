use crate::src::ffi::libc::getsockopt;
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__gid_t, __socklen_t, __uid_t, gid_t, socklen_t, uid_t};
pub use crate::src::shared::socket::SOL_SOCKET;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ucred {
    pub pid: pid_t,
    pub uid: uid_t,
    pub gid: gid_t,
}

pub const SO_PEERCRED: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn getpeereid(
    mut s: ::core::ffi::c_int,
    mut uid: *mut uid_t,
    mut gid: *mut gid_t,
) -> ::core::ffi::c_int {
    let mut uc: ucred = ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len: socklen_t = ::core::mem::size_of::<ucred>() as socklen_t;
    if getsockopt(
        s,
        SOL_SOCKET,
        SO_PEERCRED,
        &raw mut uc as *mut ::core::ffi::c_void,
        &raw mut len,
    ) == -(1 as ::core::ffi::c_int)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *uid = uc.uid;
    *gid = uc.gid;
    return 0 as ::core::ffi::c_int;
}

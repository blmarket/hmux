//! Authoritative socket declarations from the translated Linux C ABI.
pub use crate::src::ffi::libc::{
    sockaddr_at, sockaddr_ax25, sockaddr_dl, sockaddr_eon, sockaddr_inarp, sockaddr_ipx,
    sockaddr_iso, sockaddr_ns, sockaddr_x25,
};
use super::abi::{uint16_t, uint32_t, uint8_t};

pub type __socket_type = ::core::ffi::c_uint;

pub const SOCK_NONBLOCK: __socket_type = 2048;

pub const SOCK_CLOEXEC: __socket_type = 524288;

pub const SOCK_PACKET: __socket_type = 10;

pub const SOCK_DCCP: __socket_type = 6;

pub const SOCK_SEQPACKET: __socket_type = 5;

pub const SOCK_RDM: __socket_type = 4;

pub const SOCK_RAW: __socket_type = 3;

pub const SOCK_DGRAM: __socket_type = 2;

pub const SOCK_STREAM: __socket_type = 1;

pub type sa_family_t = ::core::ffi::c_ushort;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [::core::ffi::c_char; 108],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: uint32_t,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct in6_addr {
    pub __in6_u: in6_addr___in6_u,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union in6_addr___in6_u {
    pub __u6_addr8: [uint8_t; 16],
    pub __u6_addr16: [uint16_t; 8],
    pub __u6_addr32: [uint32_t; 4],
}

pub type in_port_t = uint16_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [::core::ffi::c_uchar; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

pub type in_addr_t = uint32_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub union __CONST_SOCKADDR_ARG {
    pub __sockaddr__: *const sockaddr,
    pub __sockaddr_at__: *const sockaddr_at,
    pub __sockaddr_ax25__: *const sockaddr_ax25,
    pub __sockaddr_dl__: *const sockaddr_dl,
    pub __sockaddr_eon__: *const sockaddr_eon,
    pub __sockaddr_in__: *const sockaddr_in,
    pub __sockaddr_in6__: *const sockaddr_in6,
    pub __sockaddr_inarp__: *const sockaddr_inarp,
    pub __sockaddr_ipx__: *const sockaddr_ipx,
    pub __sockaddr_iso__: *const sockaddr_iso,
    pub __sockaddr_ns__: *const sockaddr_ns,
    pub __sockaddr_un__: *const sockaddr_un,
    pub __sockaddr_x25__: *const sockaddr_x25,
}

pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;

pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;

pub const PF_UNSPEC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const SOL_SOCKET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub union __SOCKADDR_ARG {
    pub __sockaddr__: *mut sockaddr,
    pub __sockaddr_at__: *mut sockaddr_at,
    pub __sockaddr_ax25__: *mut sockaddr_ax25,
    pub __sockaddr_dl__: *mut sockaddr_dl,
    pub __sockaddr_eon__: *mut sockaddr_eon,
    pub __sockaddr_in__: *mut sockaddr_in,
    pub __sockaddr_in6__: *mut sockaddr_in6,
    pub __sockaddr_inarp__: *mut sockaddr_inarp,
    pub __sockaddr_ipx__: *mut sockaddr_ipx,
    pub __sockaddr_iso__: *mut sockaddr_iso,
    pub __sockaddr_ns__: *mut sockaddr_ns,
    pub __sockaddr_un__: *mut sockaddr_un,
    pub __sockaddr_x25__: *mut sockaddr_x25,
}

//! Authoritative errno declarations from the translated Linux C ABI.

pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const ENAMETOOLONG: ::core::ffi::c_int = 36 as ::core::ffi::c_int;

pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;

pub const ECHILD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;

pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;

pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;

pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;

pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;

pub const EBADMSG: ::core::ffi::c_int = 74 as ::core::ffi::c_int;

pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;

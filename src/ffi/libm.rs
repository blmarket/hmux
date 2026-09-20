//! Foreign declarations supplied by libm.

extern "C" {
    pub fn fabs(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn fmod(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn round(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
}

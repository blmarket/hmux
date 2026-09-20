//! Foreign declarations supplied by utempter.

extern "C" {
    pub fn utempter_add_record(
        master_fd: ::core::ffi::c_int,
        hostname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn utempter_remove_record(master_fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
}

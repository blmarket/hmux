extern "C" {
    fn forkpty(
        __amaster: *mut ::core::ffi::c_int,
        __name: *mut ::core::ffi::c_char,
        __termp: *const termios,
        __winp: *const winsize,
    ) -> ::core::ffi::c_int;
}
pub type __pid_t = ::core::ffi::c_int;
pub type pid_t = __pid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
pub type cc_t = ::core::ffi::c_uchar;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c2rust_unnamed: C2RustUnnamed_0,
    pub c2rust_unnamed_0: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
#[no_mangle]
pub unsafe extern "C" fn getptmfd() -> ::core::ffi::c_int {
    return 2147483647 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn fdforkpty(
    mut ptmfd: ::core::ffi::c_int,
    mut master: *mut ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
    mut tio: *mut termios,
    mut ws: *mut winsize,
) -> pid_t {
    return forkpty(master, name, tio, ws) as pid_t;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

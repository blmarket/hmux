//! Authoritative terminal ABI declarations.

use super::abi::{cc_t, speed_t, tcflag_t};

// Adapt the translated model's termios layout at the runtime boundary. These
// functions retain the errno/return convention until the model is migrated.
pub(crate) unsafe fn read_attributes(fd: i32, attributes: &mut termios) -> i32 {
    if fd < 0 {
        *libc::__errno_location() = libc::EBADF;
        return -1;
    }
    crate::src::reactor::io_status(
        hmux_rt::unix::terminal_attributes(std::os::fd::BorrowedFd::borrow_raw(fd)).map(|native| {
            *attributes = termios {
                c_iflag: native.c_iflag,
                c_oflag: native.c_oflag,
                c_cflag: native.c_cflag,
                c_lflag: native.c_lflag,
                c_line: native.c_line,
                c_cc: native.c_cc,
                c2rust_unnamed: termios_input_speed {
                    c_ispeed: native.c_ispeed,
                },
                c2rust_unnamed_0: termios_output_speed {
                    c_ospeed: native.c_ospeed,
                },
            };
        }),
    )
}

pub(crate) unsafe fn set_attributes(fd: i32, action: i32, attributes: &termios) -> i32 {
    if fd < 0 {
        *libc::__errno_location() = libc::EBADF;
        return -1;
    }
    let mut native: libc::termios = std::mem::zeroed();
    native.c_iflag = attributes.c_iflag;
    native.c_oflag = attributes.c_oflag;
    native.c_cflag = attributes.c_cflag;
    native.c_lflag = attributes.c_lflag;
    native.c_line = attributes.c_line;
    native.c_cc = attributes.c_cc;
    native.c_ispeed = attributes.c2rust_unnamed.c_ispeed;
    native.c_ospeed = attributes.c2rust_unnamed_0.c_ospeed;
    crate::src::reactor::io_status(hmux_rt::unix::set_terminal_attributes(
        std::os::fd::BorrowedFd::borrow_raw(fd),
        action,
        &native,
    ))
}

pub(crate) unsafe fn read_size(fd: i32, size: &mut super::posix_terminal::winsize) -> i32 {
    if fd < 0 {
        *libc::__errno_location() = libc::EBADF;
        return -1;
    }
    crate::src::reactor::io_status(
        hmux_rt::unix::terminal_size(std::os::fd::BorrowedFd::borrow_raw(fd)).map(|native| {
            *size = super::posix_terminal::winsize {
                ws_row: native.ws_row,
                ws_col: native.ws_col,
                ws_xpixel: native.ws_xpixel,
                ws_ypixel: native.ws_ypixel,
            };
        }),
    )
}

pub(crate) unsafe fn set_size(fd: i32, size: &super::posix_terminal::winsize) -> i32 {
    if fd < 0 {
        *libc::__errno_location() = libc::EBADF;
        return -1;
    }
    crate::src::reactor::io_status(hmux_rt::unix::set_terminal_size(
        std::os::fd::BorrowedFd::borrow_raw(fd),
        &libc::winsize {
            ws_row: size.ws_row,
            ws_col: size.ws_col,
            ws_xpixel: size.ws_xpixel,
            ws_ypixel: size.ws_ypixel,
        },
    ))
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c2rust_unnamed: termios_input_speed,
    pub c2rust_unnamed_0: termios_output_speed,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union termios_output_speed {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}

impl Default for termios_output_speed {
    fn default() -> Self {
        Self { __ospeed: 0 }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union termios_input_speed {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}

impl Default for termios_input_speed {
    fn default() -> Self {
        Self { __ispeed: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, offset_of, size_of};

    #[test]
    fn termios_layout_matches_translated_c_baseline() {
        assert_eq!(size_of::<termios_input_speed>(), 4);
        assert_eq!(align_of::<termios_input_speed>(), 4);
        assert_eq!(size_of::<termios_output_speed>(), 4);
        assert_eq!(align_of::<termios_output_speed>(), 4);
        assert_eq!(size_of::<termios>(), 60);
        assert_eq!(align_of::<termios>(), 4);
        assert_eq!(offset_of!(termios, c_cc), 17);
        assert_eq!(offset_of!(termios, c2rust_unnamed), 52);
        assert_eq!(offset_of!(termios, c2rust_unnamed_0), 56);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct termtype {
    pub term_names: *mut ::core::ffi::c_char,
    pub str_table: *mut ::core::ffi::c_char,
    pub Booleans: *mut ::core::ffi::c_char,
    pub Numbers: *mut ::core::ffi::c_short,
    pub Strings: *mut *mut ::core::ffi::c_char,
    pub ext_str_table: *mut ::core::ffi::c_char,
    pub ext_Names: *mut *mut ::core::ffi::c_char,
    pub num_Booleans: ::core::ffi::c_ushort,
    pub num_Numbers: ::core::ffi::c_ushort,
    pub num_Strings: ::core::ffi::c_ushort,
    pub ext_Booleans: ::core::ffi::c_ushort,
    pub ext_Numbers: ::core::ffi::c_ushort,
    pub ext_Strings: ::core::ffi::c_ushort,
}

pub type TERMTYPE = termtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct term {
    pub type_0: TERMTYPE,
}

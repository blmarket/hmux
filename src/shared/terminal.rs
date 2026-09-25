//! Authoritative terminal ABI declarations.

use super::abi::{cc_t, speed_t, tcflag_t};

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

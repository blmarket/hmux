pub use crate::src::shared::limits::{__SCHAR_MAX__, UCHAR_MAX};
use crate::src::shared::abi::*;
extern "C" {
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
}
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;

pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_SP: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VIS_SAFE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const VIS_ALL: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const VIS_NOSLASH: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VIS_GLOB: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn vis(
    mut dst: *mut ::core::ffi::c_char,
    mut c: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
    mut nextc: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    if (c == '\\' as i32 || flag & VIS_ALL == 0 as ::core::ffi::c_int)
        && (c as u_int <= UCHAR_MAX as u_int
            && c as u_char as ::core::ffi::c_int & !(0x7f as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
            && (c != '*' as i32 && c != '?' as i32 && c != '[' as i32 && c != '#' as i32
                || flag & VIS_GLOB == 0 as ::core::ffi::c_int)
            && *(*__ctype_b_loc()).offset(c as u_char as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            || flag & VIS_SP == 0 as ::core::ffi::c_int && c == ' ' as i32
            || flag & VIS_TAB == 0 as ::core::ffi::c_int && c == '\t' as i32
            || flag & VIS_NL == 0 as ::core::ffi::c_int && c == '\n' as i32
            || flag & VIS_SAFE != 0
                && (c == '\u{8}' as i32
                    || c == '\u{7}' as i32
                    || c == '\r' as i32
                    || *(*__ctype_b_loc()).offset(c as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0))
    {
        if c == '"' as i32 && flag & VIS_DQ != 0 as ::core::ffi::c_int
            || c == '\\' as i32 && flag & VIS_NOSLASH == 0 as ::core::ffi::c_int
        {
            let fresh0 = dst;
            dst = dst.offset(1);
            *fresh0 = '\\' as i32 as ::core::ffi::c_char;
        }
        let fresh1 = dst;
        dst = dst.offset(1);
        *fresh1 = c as ::core::ffi::c_char;
        *dst = '\0' as i32 as ::core::ffi::c_char;
        return dst;
    }
    if flag & VIS_CSTYLE != 0 {
        match c {
            10 => {
                current_block = 17172381846212826752;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            13 => {
                current_block = 15896914202393627355;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            8 => {
                current_block = 16114436840521083897;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            7 => {
                current_block = 12286141215944697742;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            11 => {
                current_block = 4505220464648351793;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            9 => {
                current_block = 286618792822599835;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            12 => {
                current_block = 1901908384453561418;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            32 => {
                current_block = 6023546525629896002;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            0 => {
                current_block = 18317758206522205410;
                match current_block {
                    17172381846212826752 => {
                        let fresh2 = dst;
                        dst = dst.offset(1);
                        *fresh2 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh3 = dst;
                        dst = dst.offset(1);
                        *fresh3 = 'n' as i32 as ::core::ffi::c_char;
                    }
                    6023546525629896002 => {
                        let fresh16 = dst;
                        dst = dst.offset(1);
                        *fresh16 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh17 = dst;
                        dst = dst.offset(1);
                        *fresh17 = 's' as i32 as ::core::ffi::c_char;
                    }
                    1901908384453561418 => {
                        let fresh14 = dst;
                        dst = dst.offset(1);
                        *fresh14 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh15 = dst;
                        dst = dst.offset(1);
                        *fresh15 = 'f' as i32 as ::core::ffi::c_char;
                    }
                    286618792822599835 => {
                        let fresh12 = dst;
                        dst = dst.offset(1);
                        *fresh12 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh13 = dst;
                        dst = dst.offset(1);
                        *fresh13 = 't' as i32 as ::core::ffi::c_char;
                    }
                    4505220464648351793 => {
                        let fresh10 = dst;
                        dst = dst.offset(1);
                        *fresh10 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh11 = dst;
                        dst = dst.offset(1);
                        *fresh11 = 'v' as i32 as ::core::ffi::c_char;
                    }
                    12286141215944697742 => {
                        let fresh8 = dst;
                        dst = dst.offset(1);
                        *fresh8 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh9 = dst;
                        dst = dst.offset(1);
                        *fresh9 = 'a' as i32 as ::core::ffi::c_char;
                    }
                    16114436840521083897 => {
                        let fresh6 = dst;
                        dst = dst.offset(1);
                        *fresh6 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh7 = dst;
                        dst = dst.offset(1);
                        *fresh7 = 'b' as i32 as ::core::ffi::c_char;
                    }
                    15896914202393627355 => {
                        let fresh4 = dst;
                        dst = dst.offset(1);
                        *fresh4 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh5 = dst;
                        dst = dst.offset(1);
                        *fresh5 = 'r' as i32 as ::core::ffi::c_char;
                    }
                    _ => {
                        let fresh18 = dst;
                        dst = dst.offset(1);
                        *fresh18 = '\\' as i32 as ::core::ffi::c_char;
                        let fresh19 = dst;
                        dst = dst.offset(1);
                        *fresh19 = '0' as i32 as ::core::ffi::c_char;
                        if nextc as u_char as ::core::ffi::c_int >= '0' as i32
                            && nextc as u_char as ::core::ffi::c_int <= '7' as i32
                        {
                            let fresh20 = dst;
                            dst = dst.offset(1);
                            *fresh20 = '0' as i32 as ::core::ffi::c_char;
                            let fresh21 = dst;
                            dst = dst.offset(1);
                            *fresh21 = '0' as i32 as ::core::ffi::c_char;
                        }
                    }
                }
                current_block = 1609532167703825882;
            }
            _ => {
                current_block = 15345278821338558188;
            }
        }
    } else {
        current_block = 15345278821338558188;
    }
    match current_block {
        15345278821338558188 => {
            if c & 0o177 as ::core::ffi::c_int == ' ' as i32
                || flag & VIS_OCTAL != 0
                || flag & VIS_GLOB != 0
                    && (c == '*' as i32 || c == '?' as i32 || c == '[' as i32 || c == '#' as i32)
            {
                let fresh22 = dst;
                dst = dst.offset(1);
                *fresh22 = '\\' as i32 as ::core::ffi::c_char;
                let fresh23 = dst;
                dst = dst.offset(1);
                *fresh23 = ((c as u_char as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
                    & 0o7 as ::core::ffi::c_int)
                    + '0' as i32) as ::core::ffi::c_char;
                let fresh24 = dst;
                dst = dst.offset(1);
                *fresh24 = ((c as u_char as ::core::ffi::c_int >> 3 as ::core::ffi::c_int
                    & 0o7 as ::core::ffi::c_int)
                    + '0' as i32) as ::core::ffi::c_char;
                let fresh25 = dst;
                dst = dst.offset(1);
                *fresh25 = ((c as u_char as ::core::ffi::c_int & 0o7 as ::core::ffi::c_int)
                    + '0' as i32) as ::core::ffi::c_char;
            } else {
                if flag & VIS_NOSLASH == 0 as ::core::ffi::c_int {
                    let fresh26 = dst;
                    dst = dst.offset(1);
                    *fresh26 = '\\' as i32 as ::core::ffi::c_char;
                }
                if c & 0o200 as ::core::ffi::c_int != 0 {
                    c &= 0o177 as ::core::ffi::c_int;
                    let fresh27 = dst;
                    dst = dst.offset(1);
                    *fresh27 = 'M' as i32 as ::core::ffi::c_char;
                }
                if *(*__ctype_b_loc()).offset(c as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _IScntrl as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    let fresh28 = dst;
                    dst = dst.offset(1);
                    *fresh28 = '^' as i32 as ::core::ffi::c_char;
                    if c == 0o177 as ::core::ffi::c_int {
                        let fresh29 = dst;
                        dst = dst.offset(1);
                        *fresh29 = '?' as i32 as ::core::ffi::c_char;
                    } else {
                        let fresh30 = dst;
                        dst = dst.offset(1);
                        *fresh30 = (c + '@' as i32) as ::core::ffi::c_char;
                    }
                } else {
                    let fresh31 = dst;
                    dst = dst.offset(1);
                    *fresh31 = '-' as i32 as ::core::ffi::c_char;
                    let fresh32 = dst;
                    dst = dst.offset(1);
                    *fresh32 = c as ::core::ffi::c_char;
                }
            }
        }
        _ => {}
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn strvis(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_char = 0;
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    start = dst;
    loop {
        c = *src;
        if !(c != 0) {
            break;
        }
        src = src.offset(1);
        dst = vis(
            dst,
            c as ::core::ffi::c_int,
            flag,
            *src as ::core::ffi::c_int,
        );
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst.offset_from(start) as ::core::ffi::c_long as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn strnvis(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut siz: size_t,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tbuf: [::core::ffi::c_char; 5] = [0; 5];
    let mut c: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    start = dst;
    end = start
        .offset(siz as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    loop {
        c = *src as ::core::ffi::c_int;
        if !(c != 0 && dst < end) {
            break;
        }
        if (c == '\\' as i32 || flag & VIS_ALL == 0 as ::core::ffi::c_int)
            && (c as u_int <= UCHAR_MAX as u_int
                && c as u_char as ::core::ffi::c_int & !(0x7f as ::core::ffi::c_int)
                    == 0 as ::core::ffi::c_int
                && (c != '*' as i32 && c != '?' as i32 && c != '[' as i32 && c != '#' as i32
                    || flag & VIS_GLOB == 0 as ::core::ffi::c_int)
                && *(*__ctype_b_loc()).offset(c as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                || flag & VIS_SP == 0 as ::core::ffi::c_int && c == ' ' as i32
                || flag & VIS_TAB == 0 as ::core::ffi::c_int && c == '\t' as i32
                || flag & VIS_NL == 0 as ::core::ffi::c_int && c == '\n' as i32
                || flag & VIS_SAFE != 0
                    && (c == '\u{8}' as i32
                        || c == '\u{7}' as i32
                        || c == '\r' as i32
                        || *(*__ctype_b_loc()).offset(c as u_char as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0))
        {
            if c == '"' as i32 && flag & VIS_DQ != 0 as ::core::ffi::c_int
                || c == '\\' as i32 && flag & VIS_NOSLASH == 0 as ::core::ffi::c_int
            {
                if dst.offset(1 as ::core::ffi::c_int as isize) >= end {
                    i = 2 as ::core::ffi::c_int;
                    break;
                } else {
                    let fresh33 = dst;
                    dst = dst.offset(1);
                    *fresh33 = '\\' as i32 as ::core::ffi::c_char;
                }
            }
            i = 1 as ::core::ffi::c_int;
            let fresh34 = dst;
            dst = dst.offset(1);
            *fresh34 = c as ::core::ffi::c_char;
            src = src.offset(1);
        } else {
            src = src.offset(1);
            i = vis(
                &raw mut tbuf as *mut ::core::ffi::c_char,
                c,
                flag,
                *src as ::core::ffi::c_int,
            )
            .offset_from(&raw mut tbuf as *mut ::core::ffi::c_char)
                as ::core::ffi::c_long as ::core::ffi::c_int;
            if dst.offset(i as isize) <= end {
                memcpy(
                    dst as *mut ::core::ffi::c_void,
                    &raw mut tbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    i as size_t,
                );
                dst = dst.offset(i as isize);
            } else {
                src = src.offset(-1);
                break;
            }
        }
    }
    if siz > 0 as size_t {
        *dst = '\0' as i32 as ::core::ffi::c_char;
    }
    if dst.offset(i as isize) > end {
        loop {
            c = *src as ::core::ffi::c_int;
            if !(c != 0) {
                break;
            }
            src = src.offset(1);
            dst = dst.offset(
                vis(
                    &raw mut tbuf as *mut ::core::ffi::c_char,
                    c,
                    flag,
                    *src as ::core::ffi::c_int,
                )
                .offset_from(&raw mut tbuf as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_long as isize,
            );
        }
    }
    return dst.offset_from(start) as ::core::ffi::c_long as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stravis(
    mut outp: *mut *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut serrno: ::core::ffi::c_int = 0;
    buf = calloc(4 as size_t, strlen(src).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    if buf.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    len = strvis(buf, src, flag);
    serrno = *__errno_location();
    *outp = realloc(
        buf as *mut ::core::ffi::c_void,
        (len + 1 as ::core::ffi::c_int) as size_t,
    ) as *mut ::core::ffi::c_char;
    if (*outp).is_null() {
        *outp = buf;
        *__errno_location() = serrno;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn strvisx(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut len: size_t,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_char = 0;
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    start = dst;
    while len > 1 as size_t {
        c = *src;
        src = src.offset(1);
        dst = vis(
            dst,
            c as ::core::ffi::c_int,
            flag,
            *src as ::core::ffi::c_int,
        );
        len = len.wrapping_sub(1);
    }
    if len != 0 {
        dst = vis(dst, *src as ::core::ffi::c_int, flag, '\0' as i32);
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst.offset_from(start) as ::core::ffi::c_long as ::core::ffi::c_int;
}

use crate::src::ffi::libc::memcmp;
use crate::src::shared::abi::*;
use crate::src::shared::grid::*;
pub use crate::src::shared::utf8::wchar_t;
use crate::src::shared::utf8::*;
pub use crate::src::shared::utf8::{
    hanguljamo_state, HANGULJAMO_STATE_CHOSEONG, HANGULJAMO_STATE_COMPOSABLE,
    HANGULJAMO_STATE_NOT_COMPOSABLE, HANGULJAMO_STATE_NOT_HANGULJAMO,
};
use crate::src::utf8::utf8_towc;

pub const HANGULJAMO_CLASS_NOT_HANGULJAMO: hanguljamo_class = 0;
pub const HANGULJAMO_CLASS_JUNGSEONG: hanguljamo_class = 2;
pub type hanguljamo_class = ::core::ffi::c_uint;
pub const HANGULJAMO_CLASS_JONGSEONG: hanguljamo_class = 3;
pub const HANGULJAMO_CLASS_CHOSEONG: hanguljamo_class = 1;
pub const HANGULJAMO_SUBCLASS_NOT_HANGULJAMO: hanguljamo_subclass = 0;
pub const HANGULJAMO_SUBCLASS_EXTENDED_OLD_JONGSEONG: hanguljamo_subclass = 11;
pub const HANGULJAMO_SUBCLASS_OLD_JONGSEONG: hanguljamo_subclass = 8;
pub const HANGULJAMO_SUBCLASS_JONGSEONG: hanguljamo_subclass = 7;
pub const HANGULJAMO_SUBCLASS_EXTENDED_OLD_JUNGSEONG: hanguljamo_subclass = 10;
pub const HANGULJAMO_SUBCLASS_OLD_JUNGSEONG: hanguljamo_subclass = 6;
pub const HANGULJAMO_SUBCLASS_JUNGSEONG_FILLER: hanguljamo_subclass = 4;
pub const HANGULJAMO_SUBCLASS_JUNGSEONG: hanguljamo_subclass = 5;
pub const HANGULJAMO_SUBCLASS_EXTENDED_OLD_CHOSEONG: hanguljamo_subclass = 9;
pub const HANGULJAMO_SUBCLASS_OLD_CHOSEONG: hanguljamo_subclass = 2;
pub const HANGULJAMO_SUBCLASS_CHOSEONG_FILLER: hanguljamo_subclass = 3;
pub const HANGULJAMO_SUBCLASS_CHOSEONG: hanguljamo_subclass = 1;
pub type hanguljamo_subclass = ::core::ffi::c_uint;
#[no_mangle]
pub unsafe extern "C" fn utf8_has_zwj(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if ((*ud).size as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        (&raw const (*ud).data as *const u_char)
            .offset((*ud).size as ::core::ffi::c_int as isize)
            .offset(-(3 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void,
        b"\xE2\x80\x8D\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_is_zwj(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        b"\xE2\x80\x8D\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_is_vs(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        b"\xEF\xB8\x8F\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_is_hangul_filler(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        b"\xE3\x85\xA4\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn utf8_regional_count(mut ud: *const utf8_data) -> u_int {
    let mut count: u_int = 0 as u_int;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i.wrapping_add(4 as u_int) <= (*ud).size as u_int {
        if (*ud).data[i as usize] as ::core::ffi::c_int == 0xf0 as ::core::ffi::c_int
            && (*ud).data[i.wrapping_add(1 as u_int) as usize] as ::core::ffi::c_int
                == 0x9f as ::core::ffi::c_int
            && (*ud).data[i.wrapping_add(2 as u_int) as usize] as ::core::ffi::c_int
                == 0x87 as ::core::ffi::c_int
            && (*ud).data[i.wrapping_add(3 as u_int) as usize] as ::core::ffi::c_int
                >= 0xa6 as ::core::ffi::c_int
            && (*ud).data[i.wrapping_add(3 as u_int) as usize] as ::core::ffi::c_int
                <= 0xbf as ::core::ffi::c_int
        {
            count = count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return count;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_should_combine(
    mut with: *const utf8_data,
    mut add: *const utf8_data,
) -> ::core::ffi::c_int {
    let mut w: wchar_t = 0;
    let mut a: wchar_t = 0;
    if utf8_towc(with, &raw mut w) as ::core::ffi::c_uint
        != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    if utf8_towc(add, &raw mut a) as ::core::ffi::c_uint
        != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    if a >= 0x1f1e6 as wchar_t
        && a <= 0x1f1ff as wchar_t
        && (w >= 0x1f1e6 as wchar_t && w <= 0x1f1ff as wchar_t)
    {
        if utf8_regional_count(with) != 1 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        if utf8_regional_count(add) != 1 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    match a {
        128075 | 128076 | 128077 | 128078 | 128079 | 128080 | 128102 | 128103 | 128104 | 128105
        | 128110 | 128112 | 128113 | 128114 | 128115 | 128116 | 128117 | 128118 | 128119
        | 128120 | 128124 | 128129 | 128130 | 128131 | 128133 | 128134 | 128135 | 128170
        | 128373 | 128378 | 128400 | 128405 | 128406 | 128581 | 128582 | 128583 | 128587
        | 128588 | 128589 | 128590 | 128591 | 128692 | 128693 | 128694 | 129318 | 129335
        | 129336 | 129337 | 129341 | 129342 | 129461 | 129462 | 129464 | 129465 | 129485
        | 129486 | 129487 | 129489 | 129490 | 129491 | 129492 | 129493 | 129494 | 129495
        | 129496 | 129497 | 129498 | 129499 | 129500 | 129501 | 129502 | 129503 => {
            if w >= 0x1f3fb as wchar_t && w <= 0x1f3ff as wchar_t {
                return 1 as ::core::ffi::c_int;
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn hanguljamo_get_subclass(mut s: *const u_char) -> hanguljamo_subclass {
    match *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
        225 => match *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            132 => {
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x80 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0x92 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_CHOSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x93 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xbf as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_OLD_CHOSEONG;
                }
            }
            133 => {
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 0x9f as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_CHOSEONG_FILLER;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 0xa0 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_JUNGSEONG_FILLER;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x80 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0x9e as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_OLD_CHOSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0xa1 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xb5 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_JUNGSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0xb6 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xbf as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_OLD_JUNGSEONG;
                }
            }
            134 => {
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x80 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xa7 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_OLD_JUNGSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0xa8 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xbf as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_JONGSEONG;
                }
            }
            135 => {
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x80 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0x82 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_JONGSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x83 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xbf as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_OLD_JONGSEONG;
                }
            }
            _ => {}
        },
        234 => {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xa5 as ::core::ffi::c_int
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0xa0 as ::core::ffi::c_int
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= 0xbc as ::core::ffi::c_int
            {
                return HANGULJAMO_SUBCLASS_EXTENDED_OLD_CHOSEONG;
            }
        }
        237 => {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0x9e as ::core::ffi::c_int
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0xb0 as ::core::ffi::c_int
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= 0xbf as ::core::ffi::c_int
            {
                return HANGULJAMO_SUBCLASS_EXTENDED_OLD_JUNGSEONG;
            }
            if !(*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 0x9f as ::core::ffi::c_int)
            {
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x80 as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0x86 as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_EXTENDED_OLD_JUNGSEONG;
                }
                if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    >= 0x8b as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        <= 0xbb as ::core::ffi::c_int
                {
                    return HANGULJAMO_SUBCLASS_EXTENDED_OLD_JONGSEONG;
                }
            }
        }
        _ => {}
    }
    return HANGULJAMO_SUBCLASS_NOT_HANGULJAMO;
}
unsafe extern "C" fn hanguljamo_get_class(mut s: *const u_char) -> hanguljamo_class {
    match hanguljamo_get_subclass(s) as ::core::ffi::c_uint {
        1 | 3 | 2 | 9 => return HANGULJAMO_CLASS_CHOSEONG,
        5 | 4 | 6 | 10 => return HANGULJAMO_CLASS_JUNGSEONG,
        7 | 8 | 11 => return HANGULJAMO_CLASS_JONGSEONG,
        0 => return HANGULJAMO_CLASS_NOT_HANGULJAMO,
        _ => {}
    }
    return HANGULJAMO_CLASS_NOT_HANGULJAMO;
}
#[no_mangle]
pub unsafe extern "C" fn hanguljamo_check_state(
    mut p_ud: *const utf8_data,
    mut ud: *const utf8_data,
) -> hanguljamo_state {
    let mut s: *const u_char = ::core::ptr::null::<u_char>();
    if (*ud).size as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        return HANGULJAMO_STATE_NOT_HANGULJAMO;
    }
    match hanguljamo_get_class(&raw const (*ud).data as *const u_char) as ::core::ffi::c_uint {
        1 => return HANGULJAMO_STATE_CHOSEONG,
        2 => {
            if ((*p_ud).size as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
                return HANGULJAMO_STATE_NOT_COMPOSABLE;
            }
            s = (&raw const (*p_ud).data as *const u_char)
                .offset((*p_ud).size as ::core::ffi::c_int as isize)
                .offset(-(3 as ::core::ffi::c_int as isize));
            if hanguljamo_get_class(s) as ::core::ffi::c_uint
                == HANGULJAMO_CLASS_CHOSEONG as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return HANGULJAMO_STATE_COMPOSABLE;
            }
            return HANGULJAMO_STATE_NOT_COMPOSABLE;
        }
        3 => {
            if ((*p_ud).size as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
                return HANGULJAMO_STATE_NOT_COMPOSABLE;
            }
            s = (&raw const (*p_ud).data as *const u_char)
                .offset((*p_ud).size as ::core::ffi::c_int as isize)
                .offset(-(3 as ::core::ffi::c_int as isize));
            if hanguljamo_get_class(s) as ::core::ffi::c_uint
                == HANGULJAMO_CLASS_JUNGSEONG as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return HANGULJAMO_STATE_COMPOSABLE;
            }
            return HANGULJAMO_STATE_NOT_COMPOSABLE;
        }
        0 => return HANGULJAMO_STATE_NOT_HANGULJAMO,
        _ => {}
    }
    return HANGULJAMO_STATE_NOT_HANGULJAMO;
}

use crate::src::shared::abi::*;
use crate::src::shared::grid::*;
extern "C" {
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strspn(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub name: *const ::core::ffi::c_char,
    pub attr: ::core::ffi::c_int,
}
#[no_mangle]
pub unsafe extern "C" fn attributes_tostring(
    mut attr: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut buf: [::core::ffi::c_char; 512] = [0; 512];
    let mut len: size_t = 0;
    if attr == 0 as ::core::ffi::c_int {
        return b"none\0" as *const u8 as *const ::core::ffi::c_char;
    }
    len = xsnprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
        b"%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
        if attr & GRID_ATTR_CHARSET != 0 {
            b"acs,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_BRIGHT != 0 {
            b"bright,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_DIM != 0 {
            b"dim,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_UNDERSCORE != 0 {
            b"underscore,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_BLINK != 0 {
            b"blink,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_REVERSE != 0 {
            b"reverse,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_HIDDEN != 0 {
            b"hidden,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_ITALICS != 0 {
            b"italics,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_STRIKETHROUGH != 0 {
            b"strikethrough,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_UNDERSCORE_2 != 0 {
            b"double-underscore,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_UNDERSCORE_3 != 0 {
            b"curly-underscore,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_UNDERSCORE_4 != 0 {
            b"dotted-underscore,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_UNDERSCORE_5 != 0 {
            b"dashed-underscore,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_OVERLINE != 0 {
            b"overline,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if attr & GRID_ATTR_NOATTR != 0 {
            b"noattr,\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
    ) as size_t;
    if len > 0 as size_t {
        buf[len.wrapping_sub(1 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut buf as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn attributes_fromstring(
    mut str: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let delimiters: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b" ,|\0");
    let mut attr: ::core::ffi::c_int = 0;
    let mut end: size_t = 0;
    let mut i: u_int = 0;
    let mut table: [C2RustUnnamed; 15] = [
        C2RustUnnamed {
            name: b"acs\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_CHARSET,
        },
        C2RustUnnamed {
            name: b"bright\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_BRIGHT,
        },
        C2RustUnnamed {
            name: b"bold\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_BRIGHT,
        },
        C2RustUnnamed {
            name: b"dim\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_DIM,
        },
        C2RustUnnamed {
            name: b"underscore\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_UNDERSCORE,
        },
        C2RustUnnamed {
            name: b"blink\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_BLINK,
        },
        C2RustUnnamed {
            name: b"reverse\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_REVERSE,
        },
        C2RustUnnamed {
            name: b"hidden\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_HIDDEN,
        },
        C2RustUnnamed {
            name: b"italics\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_ITALICS,
        },
        C2RustUnnamed {
            name: b"strikethrough\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_STRIKETHROUGH,
        },
        C2RustUnnamed {
            name: b"double-underscore\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_UNDERSCORE_2,
        },
        C2RustUnnamed {
            name: b"curly-underscore\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_UNDERSCORE_3,
        },
        C2RustUnnamed {
            name: b"dotted-underscore\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_UNDERSCORE_4,
        },
        C2RustUnnamed {
            name: b"dashed-underscore\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_UNDERSCORE_5,
        },
        C2RustUnnamed {
            name: b"overline\0" as *const u8 as *const ::core::ffi::c_char,
            attr: GRID_ATTR_OVERLINE,
        },
    ];
    if *str as ::core::ffi::c_int == '\0' as i32
        || strcspn(str, &raw const delimiters as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_ulong
    {
        return -(1 as ::core::ffi::c_int);
    }
    if !strchr(
        &raw const delimiters as *const ::core::ffi::c_char,
        *str.offset(strlen(str).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
    )
    .is_null()
    {
        return -(1 as ::core::ffi::c_int);
    }
    if strcasecmp(str, b"default\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(str, b"none\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    attr = 0 as ::core::ffi::c_int;
    loop {
        end = strcspn(str, &raw const delimiters as *const ::core::ffi::c_char) as size_t;
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[C2RustUnnamed; 15]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed>() as usize)
        {
            if !(end != strlen(table[i as usize].name)) {
                if strncasecmp(str, table[i as usize].name, end) == 0 as ::core::ffi::c_int {
                    attr |= table[i as usize].attr;
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
        if i as usize
            == (::core::mem::size_of::<[C2RustUnnamed; 15]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed>() as usize)
        {
            return -(1 as ::core::ffi::c_int);
        }
        str = str.offset(end.wrapping_add(strspn(
            str.offset(end as isize),
            &raw const delimiters as *const ::core::ffi::c_char,
        ) as size_t) as isize);
        if !(*str as ::core::ffi::c_int != '\0' as i32) {
            break;
        }
    }
    return attr;
}

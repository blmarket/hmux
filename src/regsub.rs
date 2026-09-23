use crate::src::ffi::libc::{memcpy, regcomp, regexec, regfree, strlen};
pub use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
};
use crate::src::xmalloc::{xmalloc, xstrdup};

unsafe fn regsub_copy(
    buf: &mut Vec<u8>,
    text: *const ::core::ffi::c_char,
    start: size_t,
    end: size_t,
) {
    buf.extend_from_slice(::core::slice::from_raw_parts(
        text.add(start).cast::<u8>(),
        end.wrapping_sub(start),
    ));
}
unsafe fn regsub_expand(
    buf: &mut Vec<u8>,
    mut with: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut m: *mut regmatch_t,
    mut n: u_int,
) {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut current_block_5: u64;
    cp = with;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\\' as i32
            && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        {
            cp = cp.offset(1);
            if *cp as ::core::ffi::c_int >= '0' as i32 && *cp as ::core::ffi::c_int <= '9' as i32 {
                i = (*cp as ::core::ffi::c_int - '0' as i32) as u_int;
                if i < n && (*m.offset(i as isize)).rm_so != (*m.offset(i as isize)).rm_eo {
                    regsub_copy(
                        buf,
                        text,
                        (*m.offset(i as isize)).rm_so as size_t,
                        (*m.offset(i as isize)).rm_eo as size_t,
                    );
                    current_block_5 = 16668937799742929182;
                } else {
                    current_block_5 = 17216689946888361452;
                }
            } else {
                current_block_5 = 17216689946888361452;
            }
        } else {
            current_block_5 = 17216689946888361452;
        }
        match current_block_5 {
            17216689946888361452 => {
                buf.push(*cp as u8);
            }
            _ => {}
        }
        cp = cp.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn regsub(
    mut pattern: *const ::core::ffi::c_char,
    mut with: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut r: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    let mut m: [regmatch_t; 10] = [regmatch_t { rm_so: 0, rm_eo: 0 }; 10];
    let mut start: ssize_t = 0;
    let mut end: ssize_t = 0;
    let mut last: ssize_t = 0;
    let mut empty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf = Vec::<u8>::new();
    if *text as ::core::ffi::c_int == '\0' as i32 {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if *pattern as ::core::ffi::c_int == '\0' as i32 {
        return xstrdup(text);
    }
    if regcomp(&raw mut r, pattern, flags) != 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    start = 0 as ssize_t;
    last = 0 as ssize_t;
    end = strlen(text) as ssize_t;
    while start <= end {
        if regexec(
            &raw mut r,
            text.offset(start as isize),
            (::core::mem::size_of::<[regmatch_t; 10]>() as size_t)
                .wrapping_div(::core::mem::size_of::<regmatch_t>() as size_t),
            &raw mut m as *mut regmatch_t,
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            regsub_copy(&mut buf, text, start as size_t, end as size_t);
            break;
        } else {
            regsub_copy(
                &mut buf,
                text,
                last as size_t,
                (m[0 as ::core::ffi::c_int as usize].rm_so as ssize_t + start) as size_t,
            );
            if *pattern as ::core::ffi::c_int == '^' as i32 {
                regsub_expand(
                    &mut buf,
                    with,
                    text.offset(start as isize),
                    &raw mut m as *mut regmatch_t,
                    (::core::mem::size_of::<[regmatch_t; 10]>() as usize)
                        .wrapping_div(::core::mem::size_of::<regmatch_t>() as usize)
                        as u_int,
                );
                last = start + m[0 as ::core::ffi::c_int as usize].rm_eo as ssize_t;
                regsub_copy(&mut buf, text, last as size_t, end as size_t);
                break;
            } else if empty != 0
                || start + m[0 as ::core::ffi::c_int as usize].rm_so as ssize_t != last
                || m[0 as ::core::ffi::c_int as usize].rm_so
                    != m[0 as ::core::ffi::c_int as usize].rm_eo
            {
                regsub_expand(
                    &mut buf,
                    with,
                    text.offset(start as isize),
                    &raw mut m as *mut regmatch_t,
                    (::core::mem::size_of::<[regmatch_t; 10]>() as usize)
                        .wrapping_div(::core::mem::size_of::<regmatch_t>() as usize)
                        as u_int,
                );
                last = start + m[0 as ::core::ffi::c_int as usize].rm_eo as ssize_t;
                start += m[0 as ::core::ffi::c_int as usize].rm_eo as ssize_t;
                empty = 0 as ::core::ffi::c_int;
            } else {
                last = start + m[0 as ::core::ffi::c_int as usize].rm_eo as ssize_t;
                start += (m[0 as ::core::ffi::c_int as usize].rm_eo as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as ssize_t;
                empty = 1 as ::core::ffi::c_int;
            }
        }
    }
    regfree(&raw mut r);
    // The exported result is freed by the format caller with libc free.
    let result = xmalloc(buf.len().wrapping_add(1)) as *mut ::core::ffi::c_char;
    memcpy(result.cast(), buf.as_ptr().cast(), buf.len());
    *result.add(buf.len()) = 0;
    result
}

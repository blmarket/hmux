use crate::src::ffi::libc::{__ctype_tolower_loc, calloc, free, memcmp, memset, strchr, strlen};
use crate::src::format::format_skip;
use crate::src::grid::grid_default_cell;
pub use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
use crate::src::shared::grid::*;
use crate::src::shared::style::*;
use crate::src::shared::utf8::*;
use crate::src::style::{style_parse, style_set};
use crate::src::utf8::{utf8_append, utf8_open, utf8_set};
use std::ffi::{CStr, CString};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzzy_char {
    pub align: style_align,
    pub ud: utf8_data,
    pub width: u_int,
    pub offset: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzzy_term {
    pub inverse: ::core::ffi::c_int,
    pub exact: ::core::ffi::c_int,
    pub prefix: ::core::ffi::c_int,
    pub suffix: ::core::ffi::c_int,
    pub text: *const ::core::ffi::c_char,
    pub len: size_t,
}
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const FUZZY_BONUS_EXACT: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const FUZZY_BONUS_PREFIX: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
pub const FUZZY_BONUS_SUFFIX: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const FUZZY_BONUS_START: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const FUZZY_BONUS_BOUNDARY: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const FUZZY_BONUS_CONSECUTIVE: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const FUZZY_PENALTY_LEADING: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FUZZY_PENALTY_LEADING_MAX: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const FUZZY_PENALTY_GAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
unsafe extern "C" fn fuzzy_is_boundary(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    static mut boundary: *const ::core::ffi::c_char =
        b" -_/.:\0" as *const u8 as *const ::core::ffi::c_char;
    if (*ud).size as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (strchr(
        boundary,
        (*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
    ) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_char_equal(
    mut a: *const utf8_data,
    mut b: *const utf8_data,
    mut fold: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if fold != 0
        && (*a).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (*b).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && ((*a).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
        && ((*b).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
    {
        return (({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int =
                        (*a).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res =
                        tolower((*a).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_tolower_loc()).offset(
                    (*a).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int;
            }
            __res
        }) == ({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int =
                        (*b).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res =
                        tolower((*b).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_tolower_loc()).offset(
                    (*b).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int;
            }
            __res
        })) as ::core::ffi::c_int;
    }
    return ((*a).size as ::core::ffi::c_int == (*b).size as ::core::ffi::c_int
        && memcmp(
            &raw const (*a).data as *const u_char as *const ::core::ffi::c_void,
            &raw const (*b).data as *const u_char as *const ::core::ffi::c_void,
            (*a).size as size_t,
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_align(mut align: style_align) -> style_align {
    if align as ::core::ffi::c_uint
        == STYLE_ALIGN_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return STYLE_ALIGN_LEFT;
    }
    return align;
}
unsafe fn fuzzy_add(
    cs: &mut Vec<fuzzy_char>,
    mut a: style_align,
    mut ud: *const utf8_data,
    mut widths: *mut u_int,
) {
    cs.push(fuzzy_char {
        align: a,
        ud: *ud,
        width: (*ud).width as u_int,
        offset: *widths.offset(a as isize),
    });
    let ref mut fresh4 = *widths.offset(a as isize);
    *fresh4 = (*fresh4).wrapping_add((*ud).width as u_int);
}
unsafe extern "C" fn fuzzy_decode_one(
    mut cp: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
    mut ud: *mut utf8_data,
) -> *const ::core::ffi::c_char {
    let mut more: utf8_state = UTF8_MORE;
    let mut start: *const ::core::ffi::c_char = cp;
    more = utf8_open(ud, *cp as u_char);
    if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
        loop {
            cp = cp.offset(1);
            if !(cp != end
                && more as ::core::ffi::c_uint
                    == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                break;
            }
            more = utf8_append(ud, *cp as u_char);
        }
        if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint {
            return cp;
        }
        cp = start;
    }
    utf8_set(ud, *cp as u_char);
    return cp.offset(1 as ::core::ffi::c_int as isize);
}
unsafe fn fuzzy_scan(
    mut text: *const ::core::ffi::c_char,
    mut widths: *mut u_int,
) -> Vec<fuzzy_char> {
    let mut cs = Vec::new();
    let mut n: u_int = 0;
    let mut leading: u_int = 0;
    let mut i: u_int = 0;
    let mut current: style_align = STYLE_ALIGN_LEFT;
    let mut sy: style = style {
        gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut cp: *const ::core::ffi::c_char = text;
    let mut textend: *const ::core::ffi::c_char = text.offset(strlen(text) as isize);
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut hash: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut bracket: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    memset(
        widths as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (::core::mem::size_of::<u_int>() as size_t).wrapping_mul(
            (STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
        ),
    );
    style_set(&raw mut sy, &raw const grid_default_cell);
    utf8_set(&raw mut hash, '#' as i32 as u_char);
    utf8_set(&raw mut bracket, '[' as i32 as u_char);
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            n = 0 as u_int;
            while *cp.offset(n as isize) as ::core::ffi::c_int == '#' as i32 {
                n = n.wrapping_add(1);
            }
            if *cp.offset(n as isize) as ::core::ffi::c_int != '[' as i32 {
                leading = if n.wrapping_rem(2 as u_int) == 0 as u_int {
                    n.wrapping_div(2 as u_int)
                } else {
                    n.wrapping_div(2 as u_int).wrapping_add(1 as u_int)
                };
                i = 0 as u_int;
                while i < leading {
                    fuzzy_add(&mut cs, current, &raw mut hash, widths);
                    i = i.wrapping_add(1);
                }
                cp = cp.offset(n as isize);
            } else {
                i = 0 as u_int;
                while i < n.wrapping_div(2 as u_int) {
                    fuzzy_add(&mut cs, current, &raw mut hash, widths);
                    i = i.wrapping_add(1);
                }
                if n.wrapping_rem(2 as u_int) == 0 as u_int {
                    fuzzy_add(&mut cs, current, &raw mut bracket, widths);
                    cp = cp.offset(n.wrapping_add(1 as u_int) as isize);
                } else {
                    end = format_skip(
                        cp.offset(n as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                        b"]\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if end.is_null() {
                        break;
                    }
                    let start = cp.add(n as usize + 1);
                    let len = end.offset_from(start) as usize;
                    let style_text =
                        CString::new(&CStr::from_ptr(start).to_bytes()[..len]).unwrap();
                    if style_parse(
                        &raw mut sy,
                        &raw const grid_default_cell,
                        style_text.as_ptr(),
                    ) == 0 as ::core::ffi::c_int
                    {
                        current = fuzzy_align(sy.align);
                    }
                    cp = end.offset(1 as ::core::ffi::c_int as isize);
                }
            }
        } else {
            cp = fuzzy_decode_one(cp, textend, &raw mut ud);
            if ud.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && (ud.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    <= 0x1f as ::core::ffi::c_int
                    || ud.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        >= 0x7f as ::core::ffi::c_int)
            {
                continue;
            }
            fuzzy_add(&mut cs, current, &raw mut ud, widths);
        }
    }
    return cs;
}
unsafe extern "C" fn fuzzy_column(
    mut fc: *const fuzzy_char,
    mut start: *const u_int,
    mut src: *const u_int,
    mut vis: *const u_int,
    mut column: *mut u_int,
) -> ::core::ffi::c_int {
    let mut a: style_align = (*fc).align;
    if (*fc).offset < *src.offset(a as isize)
        || (*fc).offset >= (*src.offset(a as isize)).wrapping_add(*vis.offset(a as isize))
    {
        return -(1 as ::core::ffi::c_int);
    }
    *column = (*start.offset(a as isize))
        .wrapping_add((*fc).offset.wrapping_sub(*src.offset(a as isize)));
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_decode(
    mut tok: *const ::core::ffi::c_char,
    mut len: size_t,
    mut out: *mut utf8_data,
) -> u_int {
    let mut cp: *const ::core::ffi::c_char = tok;
    let mut end: *const ::core::ffi::c_char = tok.offset(len as isize);
    let mut n: u_int = 0 as u_int;
    while cp != end {
        let fresh2 = n;
        n = n.wrapping_add(1);
        cp = fuzzy_decode_one(cp, end, out.offset(fresh2 as isize) as *mut utf8_data);
    }
    return n;
}
unsafe extern "C" fn fuzzy_score_positions(
    mut pos: *const u_int,
    mut npos: u_int,
    mut cs: *const fuzzy_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut gap: u_int = 0;
    let mut span: u_int = 0;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if npos == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if *pos.offset(0 as ::core::ffi::c_int as isize) == 0 as u_int {
        score += FUZZY_BONUS_START;
    } else {
        if fuzzy_is_boundary(
            &raw const (*cs.offset(
                (*pos.offset(0 as ::core::ffi::c_int as isize)).wrapping_sub(1 as u_int) as isize,
            ))
            .ud,
        ) != 0
        {
            score += FUZZY_BONUS_BOUNDARY;
        }
        if *pos.offset(0 as ::core::ffi::c_int as isize) < FUZZY_PENALTY_LEADING_MAX as u_int {
            score = (score as u_int).wrapping_sub(
                (*pos.offset(0 as ::core::ffi::c_int as isize))
                    .wrapping_mul(FUZZY_PENALTY_LEADING as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        } else {
            score -= FUZZY_PENALTY_LEADING_MAX * FUZZY_PENALTY_LEADING;
        }
    }
    i = 1 as u_int;
    while i < npos {
        if *pos.offset(i as isize)
            == (*pos.offset(i.wrapping_sub(1 as u_int) as isize)).wrapping_add(1 as u_int)
        {
            score += FUZZY_BONUS_CONSECUTIVE;
        } else if fuzzy_is_boundary(
            &raw const (*cs.offset((*pos.offset(i as isize)).wrapping_sub(1 as u_int) as isize)).ud,
        ) != 0
        {
            score += FUZZY_BONUS_BOUNDARY;
        }
        i = i.wrapping_add(1);
    }
    span = (*pos.offset(npos.wrapping_sub(1 as u_int) as isize))
        .wrapping_sub(*pos.offset(0 as ::core::ffi::c_int as isize))
        .wrapping_add(1 as u_int);
    gap = span.wrapping_sub(npos);
    score = (score as u_int).wrapping_sub(gap.wrapping_mul(FUZZY_PENALTY_GAP as u_int))
        as ::core::ffi::c_int as ::core::ffi::c_int;
    return score;
}
unsafe extern "C" fn fuzzy_match_fuzzy(
    mut tok: *const utf8_data,
    mut toklen: u_int,
    mut cs: *mut fuzzy_char,
    mut ncs: u_int,
    mut fold: ::core::ffi::c_int,
    mut score: *mut ::core::ffi::c_int,
    mut matched: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pi: u_int = 0;
    let mut ci: u_int = 0;
    let mut pos: Vec<u_int>;
    let mut found: ::core::ffi::c_int = 0;
    let mut value: ::core::ffi::c_int = 0;
    if toklen == 0 as u_int || ncs == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    pos = vec![0; toklen as usize];
    ci = 0 as u_int;
    pi = 0 as u_int;
    while pi < toklen {
        while ci != ncs
            && fuzzy_char_equal(
                tok.offset(pi as isize) as *const utf8_data,
                &raw mut (*cs.offset(ci as isize)).ud,
                fold,
            ) == 0
        {
            ci = ci.wrapping_add(1);
        }
        if ci == ncs {
            return 0 as ::core::ffi::c_int;
        }
        let fresh1 = ci;
        ci = ci.wrapping_add(1);
        pos[pi as usize] = fresh1;
        pi = pi.wrapping_add(1);
    }
    ci = pos[toklen.wrapping_sub(1 as u_int) as usize];
    pi = toklen;
    while pi > 0 as u_int {
        found = 0 as ::core::ffi::c_int;
        loop {
            if fuzzy_char_equal(
                tok.offset(pi.wrapping_sub(1 as u_int) as isize) as *const utf8_data,
                &raw mut (*cs.offset(ci as isize)).ud,
                fold,
            ) != 0
            {
                pos[pi.wrapping_sub(1 as u_int) as usize] = ci;
                found = 1 as ::core::ffi::c_int;
                break;
            } else {
                if ci == 0 as u_int {
                    break;
                }
                ci = ci.wrapping_sub(1);
            }
        }
        if found == 0 {
            return 0 as ::core::ffi::c_int;
        }
        if pi != 1 as u_int {
            ci = ci.wrapping_sub(1);
        }
        pi = pi.wrapping_sub(1);
    }
    value = fuzzy_score_positions(pos.as_ptr(), toklen, cs);
    *score += value;
    pi = 0 as u_int;
    while pi < toklen {
        *matched.offset(pos[pi as usize] as isize) = 1 as ::core::ffi::c_char;
        pi = pi.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_score_exact(
    mut start: u_int,
    mut toklen: u_int,
    mut ncs: u_int,
    mut cs: *const fuzzy_char,
    mut prefix: ::core::ffi::c_int,
    mut suffix: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut score: ::core::ffi::c_int = 0;
    score = (FUZZY_BONUS_EXACT as u_int)
        .wrapping_add(toklen.wrapping_mul(FUZZY_BONUS_CONSECUTIVE as u_int))
        as ::core::ffi::c_int;
    if prefix != 0 {
        score += FUZZY_BONUS_PREFIX;
    }
    if suffix != 0 {
        score += FUZZY_BONUS_SUFFIX;
    }
    if start == 0 as u_int {
        score += FUZZY_BONUS_START;
    } else if fuzzy_is_boundary(&raw const (*cs.offset(start.wrapping_sub(1 as u_int) as isize)).ud)
        != 0
    {
        score += FUZZY_BONUS_BOUNDARY;
    }
    if start < FUZZY_PENALTY_LEADING_MAX as u_int {
        score = (score as u_int).wrapping_sub(start.wrapping_mul(FUZZY_PENALTY_LEADING as u_int))
            as ::core::ffi::c_int as ::core::ffi::c_int;
    } else {
        score -= FUZZY_PENALTY_LEADING_MAX * FUZZY_PENALTY_LEADING;
    }
    if prefix == 0 && suffix == 0 {
        score = (score as u_int).wrapping_sub(ncs.wrapping_sub(start.wrapping_add(toklen)))
            as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    return score;
}
unsafe extern "C" fn fuzzy_match_exact(
    mut tok: *const utf8_data,
    mut toklen: u_int,
    mut cs: *mut fuzzy_char,
    mut ncs: u_int,
    mut fold: ::core::ffi::c_int,
    mut prefix: ::core::ffi::c_int,
    mut suffix: ::core::ffi::c_int,
    mut score: *mut ::core::ffi::c_int,
    mut matched: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut best: u_int = 0 as u_int;
    let mut ok: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut value: ::core::ffi::c_int = 0;
    let mut bestscore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if toklen == 0 as u_int || toklen > ncs {
        return 0 as ::core::ffi::c_int;
    }
    if prefix != 0 && suffix != 0 {
        if toklen != ncs {
            return 0 as ::core::ffi::c_int;
        }
        start = 0 as u_int;
        end = 1 as u_int;
    } else if prefix != 0 {
        start = 0 as u_int;
        end = 1 as u_int;
    } else if suffix != 0 {
        start = ncs.wrapping_sub(toklen);
        end = start.wrapping_add(1 as u_int);
    } else {
        start = 0 as u_int;
        end = ncs.wrapping_sub(toklen).wrapping_add(1 as u_int);
    }
    i = start;
    while i < end {
        ok = 1 as ::core::ffi::c_int;
        j = 0 as u_int;
        while j < toklen {
            if fuzzy_char_equal(
                tok.offset(j as isize) as *const utf8_data,
                &raw mut (*cs.offset(i.wrapping_add(j) as isize)).ud,
                fold,
            ) == 0
            {
                ok = 0 as ::core::ffi::c_int;
                break;
            } else {
                j = j.wrapping_add(1);
            }
        }
        if !(ok == 0) {
            value = fuzzy_score_exact(i, toklen, ncs, cs, prefix, suffix);
            if found == 0 || value > bestscore {
                found = 1 as ::core::ffi::c_int;
                best = i;
                bestscore = value;
            }
        }
        i = i.wrapping_add(1);
    }
    if found == 0 {
        return 0 as ::core::ffi::c_int;
    }
    *score += bestscore;
    if !matched.is_null() {
        i = 0 as u_int;
        while i < toklen {
            *matched.offset(best.wrapping_add(i) as isize) = 1 as ::core::ffi::c_char;
            i = i.wrapping_add(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_parse_term(
    mut start: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
    mut term: *mut fuzzy_term,
) -> ::core::ffi::c_int {
    memset(
        term as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<fuzzy_term>() as size_t,
    );
    if start == end {
        return 0 as ::core::ffi::c_int;
    }
    if *start as ::core::ffi::c_int == '!' as i32 {
        (*term).inverse = 1 as ::core::ffi::c_int;
        start = start.offset(1);
    }
    if start == end {
        return 0 as ::core::ffi::c_int;
    }
    if *start as ::core::ffi::c_int == '\'' as i32 {
        (*term).exact = 1 as ::core::ffi::c_int;
        start = start.offset(1);
    } else if *start as ::core::ffi::c_int == '^' as i32 {
        (*term).exact = 1 as ::core::ffi::c_int;
        (*term).prefix = 1 as ::core::ffi::c_int;
        start = start.offset(1);
    }
    if start == end {
        return 0 as ::core::ffi::c_int;
    }
    if *end.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == '$' as i32 {
        (*term).exact = 1 as ::core::ffi::c_int;
        (*term).suffix = 1 as ::core::ffi::c_int;
        end = end.offset(-1);
    }
    if start == end {
        return 0 as ::core::ffi::c_int;
    }
    if (*term).inverse != 0 {
        (*term).exact = 1 as ::core::ffi::c_int;
    }
    (*term).text = start;
    (*term).len = end.offset_from(start) as ::core::ffi::c_long as size_t;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_match_term(
    mut term: *const fuzzy_term,
    mut tok: *mut utf8_data,
    mut cs: *mut fuzzy_char,
    mut ncs: u_int,
    mut fold: ::core::ffi::c_int,
    mut score: *mut ::core::ffi::c_int,
    mut matched: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut toklen: u_int = 0;
    let mut value: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matched_term: ::core::ffi::c_int = 0;
    toklen = fuzzy_decode((*term).text, (*term).len, tok);
    if (*term).exact != 0 {
        matched_term = fuzzy_match_exact(
            tok,
            toklen,
            cs,
            ncs,
            fold,
            (*term).prefix,
            (*term).suffix,
            &raw mut value,
            if (*term).inverse != 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                matched
            },
        );
    } else {
        matched_term = fuzzy_match_fuzzy(
            tok,
            toklen,
            cs,
            ncs,
            fold,
            &raw mut value,
            if (*term).inverse != 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                matched
            },
        );
    }
    if (*term).inverse != 0 {
        return (matched_term == 0) as ::core::ffi::c_int;
    }
    if matched_term == 0 {
        return 0 as ::core::ffi::c_int;
    }
    *score += value;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_match_group(
    mut start: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
    mut tok: *mut utf8_data,
    mut cs: *mut fuzzy_char,
    mut ncs: u_int,
    mut fold: ::core::ffi::c_int,
    mut score: *mut ::core::ffi::c_int,
    mut matched: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cp: *const ::core::ffi::c_char = start;
    let mut sp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut term: fuzzy_term = fuzzy_term {
        inverse: 0,
        exact: 0,
        prefix: 0,
        suffix: 0,
        text: ::core::ptr::null::<::core::ffi::c_char>(),
        len: 0,
    };
    let mut any: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    *score = 0 as ::core::ffi::c_int;
    while cp != end {
        while cp != end && *cp as ::core::ffi::c_int == ' ' as i32 {
            cp = cp.offset(1);
        }
        if cp == end {
            break;
        }
        sp = cp;
        while cp != end && *cp as ::core::ffi::c_int != ' ' as i32 {
            cp = cp.offset(1);
        }
        if fuzzy_parse_term(sp, cp, &raw mut term) == 0 {
            return 0 as ::core::ffi::c_int;
        }
        any = 1 as ::core::ffi::c_int;
        if fuzzy_match_term(&raw mut term, tok, cs, ncs, fold, score, matched) == 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    return any;
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match(
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut width: u_int,
    mut score: *mut u_int,
) -> *mut bitstr_t {
    let mut cs: Vec<fuzzy_char>;
    let mut matched: Vec<::core::ffi::c_char>;
    let mut best: Vec<::core::ffi::c_char>;
    let mut tok: Vec<utf8_data>;
    let mut mask: *mut bitstr_t = ::core::ptr::null_mut::<bitstr_t>();
    let ncs: u_int;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut column: u_int = 0;
    let mut widths: [u_int; 5] = [0; 5];
    let mut start: [u_int; 5] = [0; 5];
    let mut src: [u_int; 5] = [0; 5];
    let mut vis: [u_int; 5] = [0; 5];
    let mut wl: u_int = 0;
    let mut wc: u_int = 0;
    let mut wr: u_int = 0;
    let mut wa: u_int = 0;
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut sp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bestscore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut groupscore: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fold: ::core::ffi::c_int = 0;
    if width == 0 as u_int {
        return ::core::ptr::null_mut::<bitstr_t>();
    }
    cp = pattern;
    while *cp as ::core::ffi::c_int == ' ' as i32 || *cp as ::core::ffi::c_int == '|' as i32 {
        cp = cp.offset(1);
    }
    if *cp as ::core::ffi::c_int == '\0' as i32 {
        if !score.is_null() {
            *score = 0 as u_int;
        }
        return calloc(
            (width.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as size_t,
            ::core::mem::size_of::<bitstr_t>() as size_t,
        ) as *mut bitstr_t;
    }
    fold = 1 as ::core::ffi::c_int;
    cp = pattern;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int >= 'A' as i32 && *cp as ::core::ffi::c_int <= 'Z' as i32 {
            fold = 0 as ::core::ffi::c_int;
            break;
        } else {
            cp = cp.offset(1);
        }
    }
    cs = fuzzy_scan(text, &raw mut widths as *mut u_int);
    ncs = cs.len() as u_int;
    matched = vec![0; ncs.max(1) as usize];
    best = vec![0; ncs.max(1) as usize];
    tok = vec![
        utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        strlen(pattern).wrapping_add(1 as size_t) as usize
    ];
    cp = pattern;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        while *cp as ::core::ffi::c_int == ' ' as i32 || *cp as ::core::ffi::c_int == '|' as i32 {
            cp = cp.offset(1);
        }
        if *cp as ::core::ffi::c_int == '\0' as i32 {
            break;
        }
        sp = cp;
        while *cp as ::core::ffi::c_int != '\0' as i32 && *cp as ::core::ffi::c_int != '|' as i32 {
            cp = cp.offset(1);
        }
        matched.fill(0);
        if fuzzy_match_group(
            sp,
            cp,
            tok.as_mut_ptr(),
            cs.as_mut_ptr(),
            ncs,
            fold,
            &raw mut groupscore,
            matched.as_mut_ptr(),
        ) != 0
        {
            if found == 0 || groupscore > bestscore {
                found = 1 as ::core::ffi::c_int;
                bestscore = groupscore;
                best.copy_from_slice(&matched);
            }
        }
    }
    drop(tok);
    if found == 0 {
        drop(best);
        drop(matched);
        drop(cs);
        return ::core::ptr::null_mut::<bitstr_t>();
    }
    wl = widths[STYLE_ALIGN_LEFT as ::core::ffi::c_int as usize];
    wc = widths[STYLE_ALIGN_CENTRE as ::core::ffi::c_int as usize];
    wr = widths[STYLE_ALIGN_RIGHT as ::core::ffi::c_int as usize];
    wa = widths[STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as usize];
    while wl.wrapping_add(wc).wrapping_add(wr) > width {
        if wc > 0 as u_int {
            wc = wc.wrapping_sub(1);
        } else if wr > 0 as u_int {
            wr = wr.wrapping_sub(1);
        } else {
            wl = wl.wrapping_sub(1);
        }
    }
    if wa > width {
        wa = width;
    }
    start[STYLE_ALIGN_LEFT as ::core::ffi::c_int as usize] = 0 as u_int;
    src[STYLE_ALIGN_LEFT as ::core::ffi::c_int as usize] = 0 as u_int;
    vis[STYLE_ALIGN_LEFT as ::core::ffi::c_int as usize] = wl;
    start[STYLE_ALIGN_RIGHT as ::core::ffi::c_int as usize] = width.wrapping_sub(wr);
    src[STYLE_ALIGN_RIGHT as ::core::ffi::c_int as usize] =
        widths[STYLE_ALIGN_RIGHT as ::core::ffi::c_int as usize].wrapping_sub(wr);
    vis[STYLE_ALIGN_RIGHT as ::core::ffi::c_int as usize] = wr;
    start[STYLE_ALIGN_CENTRE as ::core::ffi::c_int as usize] = wl
        .wrapping_add(
            width
                .wrapping_sub(wr)
                .wrapping_sub(wl)
                .wrapping_div(2 as u_int),
        )
        .wrapping_sub(wc.wrapping_div(2 as u_int));
    src[STYLE_ALIGN_CENTRE as ::core::ffi::c_int as usize] = widths
        [STYLE_ALIGN_CENTRE as ::core::ffi::c_int as usize]
        .wrapping_div(2 as u_int)
        .wrapping_sub(wc.wrapping_div(2 as u_int));
    vis[STYLE_ALIGN_CENTRE as ::core::ffi::c_int as usize] = wc;
    start[STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as usize] =
        width.wrapping_sub(wa).wrapping_div(2 as u_int);
    src[STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as usize] = 0 as u_int;
    vis[STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as usize] = wa;
    mask = calloc(
        (width.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<bitstr_t>() as size_t,
    ) as *mut bitstr_t;
    i = 0 as u_int;
    while i < ncs {
        if best[i as usize] != 0 {
            if !(fuzzy_column(
                cs.as_ptr().add(i as usize),
                &raw mut start as *mut u_int,
                &raw mut src as *mut u_int,
                &raw mut vis as *mut u_int,
                &raw mut column,
            ) != 0 as ::core::ffi::c_int)
            {
                j = 0 as u_int;
                while j < cs[i as usize].width && column.wrapping_add(j) < width {
                    let ref mut fresh0 =
                        *mask.offset((column.wrapping_add(j) >> 3 as ::core::ffi::c_int) as isize);
                    *fresh0 = (*fresh0 as ::core::ffi::c_int
                        | (1 as ::core::ffi::c_int) << (column.wrapping_add(j) & 0x7 as u_int))
                        as bitstr_t;
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    drop(best);
    drop(matched);
    drop(cs);
    if !score.is_null() {
        *score = if bestscore < 0 as ::core::ffi::c_int {
            0 as u_int
        } else {
            bestscore as u_int
        };
    }
    return mask;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracketed_style_controls_fuzzy_match_columns() {
        unsafe {
            let pattern = c"R";
            let right = c"L#[align=right]R";
            let mask = fuzzy_match(pattern.as_ptr(), right.as_ptr(), 8, std::ptr::null_mut());
            assert!(!mask.is_null());
            assert_eq!(*mask, 1 << 7);
            free(mask.cast());

            let invalid = c"L#[align=bogus]R";
            let mask = fuzzy_match(pattern.as_ptr(), invalid.as_ptr(), 8, std::ptr::null_mut());
            assert!(!mask.is_null());
            assert_eq!(*mask, 1 << 1);
            free(mask.cast());

            let incomplete = c"L#[align=right R";
            let mask = fuzzy_match(
                pattern.as_ptr(),
                incomplete.as_ptr(),
                8,
                std::ptr::null_mut(),
            );
            assert!(mask.is_null());
        }
    }

    #[test]
    fn fuzzy_match_backtracks_and_resets_between_alternatives() {
        unsafe {
            let mask = fuzzy_match(
                c"q|abc".as_ptr(),
                c"aabcbc".as_ptr(),
                6,
                std::ptr::null_mut(),
            );
            assert!(!mask.is_null());
            assert_eq!(*mask, 0b001110);
            free(mask.cast());

            let mask = fuzzy_match(
                c"abc|q".as_ptr(),
                c"aabcbc".as_ptr(),
                6,
                std::ptr::null_mut(),
            );
            assert!(!mask.is_null());
            assert_eq!(*mask, 0b001110);
            free(mask.cast());

            assert!(
                fuzzy_match(c"abc".as_ptr(), c"abx".as_ptr(), 3, std::ptr::null_mut()).is_null()
            );
        }
    }

    #[test]
    fn fuzzy_match_crosses_scan_growth_boundary_and_stops_at_nul() {
        let text = format!("{}zy\0q", "a".repeat(63));
        unsafe {
            let mask = fuzzy_match(
                c"zy".as_ptr(),
                text.as_ptr().cast(),
                65,
                std::ptr::null_mut(),
            );
            assert!(!mask.is_null());
            assert_eq!(
                std::slice::from_raw_parts(mask.cast::<u8>(), 9),
                &[0, 0, 0, 0, 0, 0, 0, 0x80, 1]
            );
            free(mask.cast());

            assert!(fuzzy_match(
                c"q".as_ptr(),
                text.as_ptr().cast(),
                65,
                std::ptr::null_mut()
            )
            .is_null());
        }
    }
}

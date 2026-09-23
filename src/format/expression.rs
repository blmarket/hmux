// Private expression parser/evaluator.  The modifier parser, loops,
// conditionals, escaping, job expansion, and recursive expansion routines
// remain in their original order. The
// facade supplies shared types, logging/state helpers, tree CRUD, callbacks,
// and job-cache lookup.
use super::*;
use std::ffi::{CStr, CString};

pub(super) unsafe extern "C" fn format_strftime(
    mut s: *mut ::core::ffi::c_char,
    mut max: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut tm: *const tm,
) -> size_t {
    return strftime(s, max, fmt, tm);
}
pub(super) unsafe fn format_quote_shell(s: *const ::core::ffi::c_char) -> CString {
    let input = CStr::from_ptr(s).to_bytes();
    let mut quoted = Vec::with_capacity(input.len().saturating_mul(2));
    const SHELL_SPECIAL: &[u8] = b"|&;<>(){}$`\\\"'*?[# =%\n\t";
    for &byte in input {
        if SHELL_SPECIAL.contains(&byte) {
            quoted.push(b'\\');
        }
        quoted.push(byte);
    }
    CString::new(quoted).expect("shell-quoted C string contains no NUL")
}
pub(super) unsafe fn format_quote_shell_single(s: *const ::core::ffi::c_char) -> CString {
    let input = CStr::from_ptr(s).to_bytes();
    let mut quoted = Vec::with_capacity(input.len().saturating_mul(4).saturating_add(2));
    quoted.push(b'\'');
    for &byte in input {
        if byte == b'\'' {
            quoted.extend_from_slice(&[b'\'', b'\\', b'\'', b'\'']);
        } else {
            quoted.push(byte);
        }
    }
    quoted.push(b'\'');
    CString::new(quoted).expect("shell-quoted C string contains no NUL")
}
pub(super) unsafe fn format_quote_style(s: *const ::core::ffi::c_char) -> CString {
    let input = CStr::from_ptr(s).to_bytes();
    let mut quoted = Vec::with_capacity(input.len().saturating_mul(2));
    for &byte in input {
        if byte == b'#' {
            quoted.push(b'#');
        }
        quoted.push(byte);
    }
    CString::new(quoted).expect("style-quoted C string contains no NUL")
}
#[no_mangle]
pub unsafe extern "C" fn format_pretty_time(
    mut t: time_t,
    mut seconds: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut now_tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut now: time_t = 0;
    let mut age: time_t = 0;
    let mut s: [::core::ffi::c_char; 9] = [0; 9];
    time(&raw mut now);
    if now < t {
        now = t;
    }
    age = now - t;
    localtime_r(&raw mut now, &raw mut now_tm);
    localtime_r(&raw mut t, &raw mut tm);
    if age < (24 as ::core::ffi::c_int * 3600 as ::core::ffi::c_int) as time_t {
        if seconds != 0 {
            strftime(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                b"%H:%M:%S\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tm,
            );
        } else {
            strftime(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                b"%H:%M\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tm,
            );
        }
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    if tm.tm_year == now_tm.tm_year && tm.tm_mon == now_tm.tm_mon
        || age
            < (28 as ::core::ffi::c_int * 24 as ::core::ffi::c_int * 3600 as ::core::ffi::c_int)
                as time_t
    {
        strftime(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
            b"%a%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tm,
        );
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    if tm.tm_year == now_tm.tm_year && tm.tm_mon < now_tm.tm_mon
        || tm.tm_year == now_tm.tm_year - 1 as ::core::ffi::c_int && tm.tm_mon > now_tm.tm_mon
    {
        strftime(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
            b"%d%b\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tm,
        );
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    strftime(
        &raw mut s as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
        b"%h%y\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut tm,
    );
    return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
}
pub(super) unsafe extern "C" fn format_relative_time(mut t: time_t) -> *mut ::core::ffi::c_char {
    let mut now: time_t = 0;
    let mut age: time_t = 0;
    let mut d: u_int = 0;
    let mut h: u_int = 0;
    let mut m: u_int = 0;
    let mut s: u_int = 0;
    let mut out: [::core::ffi::c_char; 32] = [0; 32];
    time(&raw mut now);
    if t > now {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if t == now {
        return xstrdup(b"0s\0" as *const u8 as *const ::core::ffi::c_char);
    }
    age = now - t;
    d = (age / 86400 as time_t) as u_int;
    h = (age % 86400 as time_t / 3600 as time_t) as u_int;
    m = (age % 3600 as time_t / 60 as time_t) as u_int;
    s = (age % 60 as time_t) as u_int;
    if d != 0 as u_int {
        if h != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%ud%uh\0" as *const u8 as *const ::core::ffi::c_char,
                d,
                h,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%ud\0" as *const u8 as *const ::core::ffi::c_char,
                d,
            );
        }
    } else if h != 0 as u_int {
        if m != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%uh%um\0" as *const u8 as *const ::core::ffi::c_char,
                h,
                m,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%uh\0" as *const u8 as *const ::core::ffi::c_char,
                h,
            );
        }
    } else if m != 0 as u_int {
        if s != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%um%us\0" as *const u8 as *const ::core::ffi::c_char,
                m,
                s,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%um\0" as *const u8 as *const ::core::ffi::c_char,
                m,
            );
        }
    } else {
        xsnprintf(
            &raw mut out as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%us\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    return xstrdup(&raw mut out as *mut ::core::ffi::c_char);
}
pub(super) unsafe extern "C" fn format_time_difference(mut t: time_t) -> *mut ::core::ffi::c_char {
    let mut now: time_t = time(::core::ptr::null_mut::<time_t>());
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut out,
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        now - t,
    );
    return out;
}
pub(super) unsafe extern "C" fn format_find(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut modifiers: uint64_t,
    mut time_format: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut fte: *const format_table_entry = ::core::ptr::null::<format_table_entry>();
    let mut value: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_find: format_entry = format_entry {
        key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        time: 0,
        cb: None,
        entry: format_entry_entry {
            rbe_left: ::core::ptr::null_mut::<format_entry>(),
            rbe_right: ::core::ptr::null_mut::<format_entry>(),
            rbe_parent: ::core::ptr::null_mut::<format_entry>(),
            rbe_color: 0,
        },
    };
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: [::core::ffi::c_char; 512] = [0; 512];
    let mut array_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut t: time_t = 0 as time_t;
    let mut tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    o = options_parse_get(
        global_options,
        key,
        &raw mut array_key,
        0 as ::core::ffi::c_int,
    );
    if o.is_null() && !(*ft).wp.is_null() {
        o = options_parse_get(
            (*(*ft).wp).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() && !(*ft).w.is_null() {
        o = options_parse_get(
            (*(*ft).w).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() {
        o = options_parse_get(
            global_w_options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() && !(*ft).s.is_null() {
        o = options_parse_get(
            (*(*ft).s).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() {
        o = options_parse_get(
            global_s_options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if !o.is_null() {
        found = options_to_string(o, array_key, 1 as ::core::ffi::c_int);
        free(array_key as *mut ::core::ffi::c_void);
    } else {
        fte = format_table_get(key);
        if !fte.is_null() {
            value = (*fte).cb.expect("non-null function pointer")(ft);
            if (*fte).type_0 as ::core::ffi::c_uint
                == FORMAT_TABLE_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
                && !value.is_null()
            {
                t = (*(value as *mut timeval)).tv_sec as time_t;
            } else {
                found = value as *mut ::core::ffi::c_char;
            }
        } else {
            fe_find.key = key as *mut ::core::ffi::c_char;
            fe = format_entry_tree_find(&raw mut (*ft).tree, &raw mut fe_find);
            if !fe.is_null() {
                if (*fe).time != 0 as time_t {
                    t = (*fe).time;
                } else {
                    if (*fe).value.is_null() && (*fe).cb.is_some() {
                        let value = (*fe).cb.expect("non-null function pointer")(ft)
                            as *mut ::core::ffi::c_char;
                        format_entry_cache_callback(fe, value);
                    }
                    found = xstrdup((*fe).value);
                }
            } else {
                if !modifiers & FORMAT_TIMESTRING as uint64_t != 0 {
                    envent = ::core::ptr::null_mut::<environ_entry>();
                    if !(*ft).s.is_null() {
                        envent = environ_find((*(*ft).s).environ, key);
                    }
                    if envent.is_null() {
                        envent = environ_find(global_environ, key);
                    }
                    if !envent.is_null() && !(*envent).value.is_null() {
                        found = xstrdup((*envent).value);
                        current_block = 11739001764845178280;
                    } else {
                        current_block = 1836292691772056875;
                    }
                } else {
                    current_block = 1836292691772056875;
                }
                match current_block {
                    11739001764845178280 => {}
                    _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }
            }
        }
    }
    if modifiers & FORMAT_TIMESTRING as uint64_t != 0 {
        if t == 0 as time_t && !found.is_null() {
            t = strtonum(
                found,
                0 as ::core::ffi::c_longlong,
                INT64_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as time_t;
            if !errstr.is_null() {
                t = 0 as time_t;
            }
            free(found as *mut ::core::ffi::c_void);
        }
        if t == 0 as time_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if modifiers & FORMAT_RELATIVE as uint64_t != 0 {
            found = format_relative_time(t);
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_DIFFERENCE != 0 {
            found = format_time_difference(t);
        } else if modifiers & FORMAT_PRETTY as uint64_t != 0 {
            found = format_pretty_time(t, 0 as ::core::ffi::c_int);
        } else {
            if !time_format.is_null() {
                localtime_r(&raw mut t, &raw mut tm);
                format_strftime(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
                    time_format,
                    &raw mut tm,
                );
            } else {
                ctime_r(&raw mut t, &raw mut s as *mut ::core::ffi::c_char);
                s[strcspn(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b"\n\0" as *const u8 as *const ::core::ffi::c_char,
                ) as usize] = '\0' as i32 as ::core::ffi::c_char;
            }
            found = xstrdup(&raw mut s as *mut ::core::ffi::c_char);
        }
        return found;
    }
    if t != 0 as time_t {
        xasprintf(
            &raw mut found,
            b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
            t as ::core::ffi::c_longlong,
        );
    } else if found.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if modifiers & FORMAT_BASENAME as uint64_t != 0 {
        saved = found;
        found = xstrdup(__xpg_basename(saved));
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_DIRNAME as uint64_t != 0 {
        saved = found;
        found = xstrdup(dirname(saved));
        free(saved as *mut ::core::ffi::c_void);
    }
    let mut quoted: Option<CString> = None;
    if modifiers & FORMAT_QUOTE_SHELL as uint64_t != 0 {
        quoted = Some(format_quote_shell(found));
    }
    if modifiers & FORMAT_QUOTE_SHELL_SQ as uint64_t != 0 {
        quoted = Some(format_quote_shell_single(
            quoted.as_ref().map_or(found, |value| value.as_ptr()),
        ));
    }
    if modifiers & FORMAT_QUOTE_STYLE as uint64_t != 0 {
        quoted = Some(format_quote_style(
            quoted.as_ref().map_or(found, |value| value.as_ptr()),
        ));
    }
    if let Some(quoted) = quoted {
        let result = xstrdup(quoted.as_ptr());
        free(found as *mut ::core::ffi::c_void);
        found = result;
    }
    if modifiers & FORMAT_QUOTE_ARGUMENTS as uint64_t != 0 {
        saved = found;
        found = args_escape(saved);
        free(saved as *mut ::core::ffi::c_void);
    }
    return found;
}
pub(super) unsafe extern "C" fn format_check_time(
    mut es: *mut format_expand_state,
    mut check: *mut u_int,
) -> ::core::ffi::c_int {
    let mut t: uint64_t = 0;
    if !check.is_null() && {
        *check = (*check).wrapping_add(1);
        (*check).wrapping_rem(FORMAT_TIME_LOOP_CHECK as u_int) != 0 as u_int
    } {
        return 1 as ::core::ffi::c_int;
    }
    t = get_timer();
    if t.wrapping_sub((*es).start_time) < FORMAT_TIME_LIMIT as uint64_t {
        return 1 as ::core::ffi::c_int;
    }
    t = t.wrapping_sub((*es).start_time);
    format_log1(
        es,
        b"format_check_time\0" as *const u8 as *const ::core::ffi::c_char,
        b"reached time limit (%llu)\0" as *const u8 as *const ::core::ffi::c_char,
        t as ::core::ffi::c_ulonglong,
    );
    return 0 as ::core::ffi::c_int;
}
unsafe fn format_unescape_cstring(
    es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    n: size_t,
) -> CString {
    let end = s.add(n);
    let mut out = Vec::with_capacity(n);
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    while s != end {
        if format_check_time(es, &raw mut check) == 0 {
            return CString::default();
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && s.offset(1 as ::core::ffi::c_int as isize) != end
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
        {
            brackets += 1;
        }
        if brackets == 0 as ::core::ffi::c_int
            && *s as ::core::ffi::c_int == '#' as i32
            && s.offset(1 as ::core::ffi::c_int as isize) != end
            && !strchr(
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
                *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            s = s.offset(1);
            out.push(*s as u8);
        } else {
            if *s as ::core::ffi::c_int == '}' as i32 {
                brackets -= 1;
            }
            out.push(*s as u8);
        }
        s = s.offset(1);
    }
    // C callers observe only the bytes before the first NUL, even if a
    // counted input contains one in the middle.
    if let Some(first_nul) = out.iter().position(|&byte| byte == 0) {
        out.truncate(first_nul);
    }
    CString::new(out).expect("unescaped C-string view has no NUL")
}
pub(super) unsafe extern "C" fn format_unescape(
    es: *mut format_expand_state,
    s: *const ::core::ffi::c_char,
    n: size_t,
) -> *mut ::core::ffi::c_char {
    xstrdup(format_unescape_cstring(es, s, n).as_ptr())
}
pub(super) unsafe extern "C" fn format_strip(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    out = xmalloc(strlen(s).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    cp = out;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        if format_check_time(es, &raw mut check) == 0 {
            free(out as *mut ::core::ffi::c_void);
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
        {
            brackets += 1;
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && !strchr(
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
                *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            if brackets != 0 as ::core::ffi::c_int {
                let fresh24 = cp;
                cp = cp.offset(1);
                *fresh24 = *s;
            }
        } else {
            if *s as ::core::ffi::c_int == '}' as i32 {
                brackets -= 1;
            }
            let fresh25 = cp;
            cp = cp.offset(1);
            *fresh25 = *s;
        }
        s = s.offset(1);
    }
    *cp = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
pub(super) unsafe extern "C" fn format_skip1(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        if !es.is_null() && format_check_time(es, &raw mut check) == 0 {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
        {
            brackets += 1;
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
            && !strchr(
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
                *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            s = s.offset(1);
        } else {
            if *s as ::core::ffi::c_int == '}' as i32 {
                brackets -= 1;
            }
            if !strchr(end, *s as ::core::ffi::c_int).is_null()
                && brackets == 0 as ::core::ffi::c_int
            {
                break;
            }
        }
        s = s.offset(1);
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn format_skip(
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    return format_skip1(::core::ptr::null_mut::<format_expand_state>(), s, end);
}
pub(super) unsafe extern "C" fn format_choose(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut left: *mut *mut ::core::ffi::c_char,
    mut right: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let cp = format_skip1(es, s, b",\0" as *const u8 as *const ::core::ffi::c_char);
    if cp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    let split = cp.offset_from(s) as usize;
    // Clone both operands before expansion: a format callback may reenter
    // the formatter or change the storage backing the original input.
    let left0 = CString::new(&CStr::from_ptr(s).to_bytes()[..split]).unwrap();
    let right0 = CStr::from_ptr(cp.add(1)).to_owned();
    *left = format_expand1(es, left0.as_ptr());
    drop(left0);
    *right = format_expand1(es, right0.as_ptr());
    return 0 as ::core::ffi::c_int;
}

// The three format loops only borrow the operands while expanding each item.
// Keep both copies alive across nested expansions, including the no-comma case.
unsafe fn format_choose_loop(
    es: *mut format_expand_state,
    fmt: *const ::core::ffi::c_char,
) -> (CString, Option<CString>) {
    let cp = format_skip1(es, fmt, b",\0".as_ptr().cast());
    if cp.is_null() {
        return (CStr::from_ptr(fmt).to_owned(), None);
    }
    let split = cp.offset_from(fmt) as usize;
    let all = CString::new(&CStr::from_ptr(fmt).to_bytes()[..split]).unwrap();
    let active = CStr::from_ptr(cp.add(1)).to_owned();
    (all, Some(active))
}
#[no_mangle]
pub unsafe extern "C" fn format_true(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if !s.is_null()
        && *s as ::core::ffi::c_int != '\0' as i32
        && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '0' as i32
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32)
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub(super) unsafe extern "C" fn format_is_end(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int == ';' as i32 || c as ::core::ffi::c_int == ':' as i32)
        as ::core::ffi::c_int;
}
pub(super) unsafe fn format_add_modifier(
    list: &mut Vec<format_modifier>,
    c: *const ::core::ffi::c_char,
    n: size_t,
    argv: Vec<CString>,
) {
    let mut fm = format_modifier {
        modifier: [0; 3],
        size: n as u_int,
        argv,
    };
    memcpy(
        fm.modifier.as_mut_ptr() as *mut ::core::ffi::c_void,
        c as *const ::core::ffi::c_void,
        n,
    );
    list.push(fm);
}
// format_expand1 returns a C allocation; copy its C-string bytes into the
// modifier owner, then release the return allocation at this boundary.
unsafe fn format_expand_modifier_arg(
    es: *mut format_expand_state,
    value: *const ::core::ffi::c_char,
) -> CString {
    let expanded = format_expand1(es, value);
    let arg = CStr::from_ptr(expanded).to_owned();
    free(expanded.cast());
    arg
}
pub(super) unsafe fn format_build_modifiers(
    mut es: *mut format_expand_state,
    mut s: *mut *const ::core::ffi::c_char,
) -> Vec<format_modifier> {
    let mut cp: *const ::core::ffi::c_char = *s;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut list: Vec<format_modifier> = Vec::new();
    let mut c: ::core::ffi::c_char = 0;
    let mut last: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"X;:\0");
    let mut argv: Vec<CString> = Vec::new();
    while *cp as ::core::ffi::c_int != '\0' as i32 && *cp as ::core::ffi::c_int != ':' as i32 {
        if *cp as ::core::ffi::c_int == ';' as i32 {
            cp = cp.offset(1);
        }
        if *cp as ::core::ffi::c_int == '\0' as i32 {
            break;
        }
        if !strchr(
            b"labdnwETSWPOVL!<>A\0" as *const u8 as *const ::core::ffi::c_char,
            *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        )
        .is_null()
            && format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            format_add_modifier(&mut list, cp, 1 as size_t, Vec::new());
            cp = cp.offset(1);
        } else if (memcmp(
            b"||\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            cp as *const ::core::ffi::c_void,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"&&\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"!!\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"!=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"==\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"<=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b">=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int)
            && format_is_end(*cp.offset(2 as ::core::ffi::c_int as isize)) != 0
        {
            format_add_modifier(&mut list, cp, 2 as size_t, Vec::new());
            cp = cp.offset(2 as ::core::ffi::c_int as isize);
        } else {
            if strchr(
                b"ImCLNPSOVst=pReqWcA\0" as *const u8 as *const ::core::ffi::c_char,
                *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
            {
                break;
            }
            c = *cp.offset(0 as ::core::ffi::c_int as isize);
            if format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0 {
                format_add_modifier(&mut list, cp, 1 as size_t, Vec::new());
                cp = cp.offset(1);
            } else {
                argv = Vec::new();
                if *(*__ctype_b_loc())
                    .offset(*cp.offset(1 as ::core::ffi::c_int as isize) as u_char
                        as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                    || *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '-' as i32
                {
                    end = format_skip1(
                        es,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                        b":;\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if end.is_null() {
                        break;
                    }
                    let value = format_unescape_cstring(
                        es,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                        end.offset_from(cp.offset(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_long as size_t,
                    );
                    argv.push(format_expand_modifier_arg(es, value.as_ptr()));
                    format_add_modifier(&mut list, &raw mut c, 1 as size_t, argv);
                    cp = end;
                } else {
                    last[0 as ::core::ffi::c_int as usize] =
                        *cp.offset(1 as ::core::ffi::c_int as isize);
                    cp = cp.offset(1);
                    loop {
                        if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == last[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                            && format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0
                        {
                            cp = cp.offset(1);
                            break;
                        } else {
                            end = format_skip1(
                                es,
                                cp.offset(1 as ::core::ffi::c_int as isize),
                                &raw mut last as *mut ::core::ffi::c_char,
                            );
                            if end.is_null() {
                                break;
                            }
                            cp = cp.offset(1);
                            let value = format_unescape_cstring(
                                es,
                                cp,
                                end.offset_from(cp) as ::core::ffi::c_long as size_t,
                            );
                            argv.push(format_expand_modifier_arg(es, value.as_ptr()));
                            cp = end;
                            if !(format_is_end(*cp.offset(0 as ::core::ffi::c_int as isize)) == 0) {
                                break;
                            }
                        }
                    }
                    format_add_modifier(&mut list, &raw mut c, 1 as size_t, argv);
                }
            }
        }
    }
    if *cp as ::core::ffi::c_int != ':' as i32 {
        drop(list);
        return Vec::new();
    }
    *s = cp.offset(1 as ::core::ffi::c_int as isize);
    return list;
}
pub(super) unsafe extern "C" fn format_match_fuzzy(
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut positions: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut bs: *mut bitstr_t = ::core::ptr::null_mut::<bitstr_t>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    width = format_width(text);
    if width == 0 as u_int {
        width = 1 as u_int;
    }
    bs = fuzzy_match(pattern, text, width, ::core::ptr::null_mut::<u_int>());
    if bs.is_null() {
        return xstrdup(if positions != 0 {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"0\0" as *const u8 as *const ::core::ffi::c_char
        });
    }
    if positions == 0 {
        free(bs as *mut ::core::ffi::c_void);
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 0 as u_int;
    while i < width {
        if !(*bs.offset((i >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (i & 0x7 as u_int)
            == 0)
        {
            if evbuffer_get_length(buffer) != 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        i = i.wrapping_add(1);
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    free(bs as *mut ::core::ffi::c_void);
    return value;
}
pub(super) unsafe extern "C" fn format_match(
    mut fm: *mut format_modifier,
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
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
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fm).argc() >= 1 as ::core::ffi::c_int {
        s = (*fm).arg(0);
    }
    if !strchr(s, 'p' as i32).is_null() {
        return format_match_fuzzy(pattern, text, 1 as ::core::ffi::c_int);
    }
    if !strchr(s, 'z' as i32).is_null() {
        return format_match_fuzzy(pattern, text, 0 as ::core::ffi::c_int);
    }
    if strchr(s, 'r' as i32).is_null() {
        if !strchr(s, 'i' as i32).is_null() {
            flags |= FNM_CASEFOLD;
        }
        if fnmatch(pattern, text, flags) != 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
        }
    } else {
        flags = REG_EXTENDED | REG_NOSUB;
        if !strchr(s, 'i' as i32).is_null() {
            flags |= REG_ICASE;
        }
        if regcomp(&raw mut r, pattern, flags) != 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if regexec(
            &raw mut r,
            text,
            0 as size_t,
            ::core::ptr::null_mut::<regmatch_t>(),
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            regfree(&raw mut r);
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
        }
        regfree(&raw mut r);
    }
    return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
}
pub(super) unsafe extern "C" fn format_sub(
    mut fm: *mut format_modifier,
    mut text: *const ::core::ffi::c_char,
    mut pattern: *const ::core::ffi::c_char,
    mut with: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = REG_EXTENDED;
    if (*fm).argc() >= 3 as ::core::ffi::c_int && !strchr((*fm).arg(2), 'i' as i32).is_null() {
        flags |= REG_ICASE;
    }
    value = regsub(pattern, with, text, flags);
    if value.is_null() {
        return xstrdup(text);
    }
    return value;
}
pub(super) unsafe extern "C" fn format_search(
    mut fm: *mut format_modifier,
    mut wp: *mut window_pane,
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regex: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*fm).argc() >= 1 as ::core::ffi::c_int {
        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
            ignore = 1 as ::core::ffi::c_int;
        }
        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
            regex = 1 as ::core::ffi::c_int;
        }
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        window_pane_search(wp, s, regex, ignore),
    );
    return value;
}
pub(super) unsafe extern "C" fn format_bool_op_1(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut not: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut result: ::core::ffi::c_int = 0;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    expanded = format_expand1(es, fmt);
    result = format_true(expanded);
    if not != 0 {
        result = (result == 0) as ::core::ffi::c_int;
    }
    free(expanded as *mut ::core::ffi::c_void);
    return xstrdup(if result != 0 {
        b"1\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b"0\0" as *const u8 as *const ::core::ffi::c_char
    });
}
pub(super) unsafe extern "C" fn format_bool_op_n(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut and: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut result: ::core::ffi::c_int = 0;
    let mut cp1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    result = if and != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    cp1 = fmt;
    while if and != 0 {
        result
    } else {
        (result == 0) as ::core::ffi::c_int
    } != 0
    {
        cp2 = format_skip1(es, cp1, b",\0" as *const u8 as *const ::core::ffi::c_char);
        let operand = if cp2.is_null() {
            CStr::from_ptr(cp1).to_owned()
        } else {
            let length = cp2.offset_from(cp1) as usize;
            CString::new(std::slice::from_raw_parts(cp1.cast::<u8>(), length))
                .expect("format operand contains no NUL before its delimiter")
        };
        expanded = format_expand1(es, operand.as_ptr());
        format_log1(
            es,
            b"format_bool_op_n\0" as *const u8 as *const ::core::ffi::c_char,
            b"operator %s has operand: %s\0" as *const u8 as *const ::core::ffi::c_char,
            if and != 0 {
                b"&&\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"||\0" as *const u8 as *const ::core::ffi::c_char
            },
            expanded,
        );
        if and != 0 {
            result = (result != 0 && format_true(expanded) != 0) as ::core::ffi::c_int;
        } else {
            result = (result != 0 || format_true(expanded) != 0) as ::core::ffi::c_int;
        }
        free(expanded as *mut ::core::ffi::c_void);
        if cp2.is_null() {
            break;
        }
        cp1 = cp2.offset(1 as ::core::ffi::c_int as isize);
    }
    return xstrdup(if result != 0 {
        b"1\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b"0\0" as *const u8 as *const ::core::ffi::c_char
    });
}
pub(super) unsafe extern "C" fn format_session_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    name = format_expand1(es, fmt);
    s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if strcmp((*s).name, name) == 0 as ::core::ffi::c_int {
            free(name as *mut ::core::ffi::c_void);
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
        }
        s = sessions_next(s);
    }
    free(name as *mut ::core::ffi::c_void);
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
}
pub(super) unsafe extern "C" fn format_loop_sessions(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: ::core::ffi::c_int = 0;
    let (all, active) = format_choose_loop(es, fmt);
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let l = sort_get_sessions(sc);
    let n = ::core::ffi::c_int::try_from(l.len()).expect("too many sessions to format");
    i = 0 as ::core::ffi::c_int;
    while i < n {
        s = l[i as usize];
        format_log1(
            es,
            b"format_loop_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            b"session loop: $%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).id,
        );
        let use_0 = if active.is_some()
            && !(*ft).c.is_null()
            && !(*(*ft).c).session.is_null()
            && (*s).id == (*(*(*ft).c).session).id
        {
            active.as_ref().unwrap().as_ptr()
        } else {
            all.as_ptr()
        };
        nft = format_create(c, item, FORMAT_NONE, (*ft).flags);
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(
            nft,
            (*ft).c,
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(next.ft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    drop(active);
    drop(all);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_window_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*ft).s.is_null() {
        format_log1(
            es,
            b"format_window_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"window name but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    name = format_expand1(es, fmt);
    wl = winlinks_minmax(&raw mut (*(*ft).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp((*(*wl).window).name, name) == 0 as ::core::ffi::c_int {
            free(name as *mut ::core::ffi::c_void);
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
        }
        wl = winlinks_next(wl);
    }
    free(name as *mut ::core::ffi::c_void);
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
}
pub(super) unsafe extern "C" fn format_add_window_neighbour(
    mut nft: *mut format_tree,
    mut wl: *mut winlink,
    mut s: *mut session,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut oval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let prefix = CStr::from_ptr(prefix).to_bytes();
    let key = CString::new([prefix, b"_window_index"].concat()).expect("C string key");
    format_add(
        nft,
        key.as_ptr(),
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    let key = CString::new([prefix, b"_window_active"].concat()).expect("C string key");
    format_add(
        nft,
        key.as_ptr(),
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (wl == (*s).curw) as ::core::ffi::c_int,
    );
    o = options_first((*(*wl).window).options);
    while !o.is_null() {
        oname = options_name(o);
        if *oname as ::core::ffi::c_int == '@' as i32 {
            let prefixed = CString::new([prefix, b"_", CStr::from_ptr(oname).to_bytes()].concat())
                .expect("C string key");
            oval = options_to_string(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                1 as ::core::ffi::c_int,
            );
            format_add(
                nft,
                prefixed.as_ptr(),
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                oval,
            );
            free(oval as *mut ::core::ffi::c_void);
        }
        o = options_next(o);
    }
}
pub(super) unsafe extern "C" fn format_loop_windows(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut s: *mut session = (*ft).s;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut i: ::core::ffi::c_int = 0;
    if s.is_null() {
        format_log1(
            es,
            b"format_loop_windows\0" as *const u8 as *const ::core::ffi::c_char,
            b"window loop but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let (all, active) = format_choose_loop(es, fmt);
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let l = sort_get_winlinks_session(s, sc);
    let n = ::core::ffi::c_int::try_from(l.len()).expect("too many winlinks in format loop");
    i = 0 as ::core::ffi::c_int;
    while i < n {
        wl = l[i as usize];
        w = (*wl).window;
        format_log1(
            es,
            b"format_loop_windows\0" as *const u8 as *const ::core::ffi::c_char,
            b"window loop: %u @%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
            (*w).id,
        );
        let use_0 = if active.is_some() && wl == (*s).curw {
            active.as_ref().unwrap().as_ptr()
        } else {
            all.as_ptr()
        };
        nft = format_create(
            c,
            item,
            (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
            (*ft).flags,
        );
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        if i > 0 as ::core::ffi::c_int && l[(i - 1 as ::core::ffi::c_int) as usize] == (*s).curw {
            format_add(
                nft,
                b"window_after_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"window_after_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (i + 1 as ::core::ffi::c_int) < n
            && l[(i + 1 as ::core::ffi::c_int) as usize] == (*s).curw
        {
            format_add(
                nft,
                b"window_before_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"window_before_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (i + 1 as ::core::ffi::c_int) < n {
            format_add_window_neighbour(
                nft,
                l[(i + 1 as ::core::ffi::c_int) as usize],
                s,
                b"next\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if i > 0 as ::core::ffi::c_int {
            format_add_window_neighbour(
                nft,
                l[(i - 1 as ::core::ffi::c_int) as usize],
                s,
                b"prev\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_defaults(nft, (*ft).c, s, wl, ::core::ptr::null_mut::<window_pane>());
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    drop(active);
    drop(all);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_loop_panes(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    if (*ft).w.is_null() {
        format_log1(
            es,
            b"format_loop_panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"pane loop but no window\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let (all, active) = format_choose_loop(es, fmt);
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let l = sort_get_panes_window((*ft).w, sc);
    let n = i32::try_from(l.len()).expect("too many panes in format loop");
    i = 0 as ::core::ffi::c_int;
    while i < n {
        wp = l[i as usize];
        format_log1(
            es,
            b"format_loop_panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"pane loop: %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
        let use_0 = if active.is_some() && wp == (*(*ft).w).active {
            active.as_ref().unwrap().as_ptr()
        } else {
            all.as_ptr()
        };
        nft = format_create(
            c,
            item,
            (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
            (*ft).flags,
        );
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    drop(active);
    drop(all);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_loop_add_option(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut buffer: *mut evbuffer,
    mut o: *mut options_entry,
    mut n: u_int,
    mut i: u_int,
) {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_array: ::core::ffi::c_int = options_is_array(o);
    format_log1(
        es,
        b"format_loop_add_option\0" as *const u8 as *const ::core::ffi::c_char,
        b"option loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    nft = format_create((*ft).client, (*ft).item, FORMAT_NONE, (*ft).flags);
    format_add(
        nft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    s = options_to_string(
        o,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    format_add(
        nft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_count\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (oe == NULL_0 as *const options_table_entry) as ::core::ffi::c_int,
    );
    if options_next(o).is_null() {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        i,
    );
    format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
    format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
    next.ft = nft;
    expanded = format_expand1(&raw mut next, fmt);
    format_free(nft);
    evbuffer_add(
        buffer,
        expanded as *const ::core::ffi::c_void,
        strlen(expanded),
    );
    free(expanded as *mut ::core::ffi::c_void);
}
pub(super) unsafe extern "C" fn format_loop_add_array_item(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut buffer: *mut evbuffer,
    mut o: *mut options_entry,
    mut a: *mut options_array_item,
    mut n: ::core::ffi::c_int,
    mut i: u_int,
) {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    array_key = options_array_item_key(a);
    format_log1(
        es,
        b"format_loop_add_array_item\0" as *const u8 as *const ::core::ffi::c_char,
        b"option loop: %s[%s]\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        array_key,
    );
    nft = format_create((*ft).client, (*ft).item, FORMAT_NONE, (*ft).flags);
    format_add(
        nft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    s = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    format_add(
        nft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        array_key,
    );
    format_add(
        nft,
        b"option_array_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        array_key,
    );
    if a == options_array_first(o) {
        format_add(
            nft,
            b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if options_array_next(a).is_null() {
        format_add(
            nft,
            b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_array_count\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (oe == NULL_0 as *const options_table_entry) as ::core::ffi::c_int,
    );
    if options_array_next(a).is_null() && options_next(o).is_null() {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        i,
    );
    format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
    format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
    next.ft = nft;
    expanded = format_expand1(&raw mut next, fmt);
    format_free(nft);
    evbuffer_add(
        buffer,
        expanded as *const ::core::ffi::c_void,
        strlen(expanded),
    );
    free(expanded as *mut ::core::ffi::c_void);
}
pub(super) unsafe extern "C" fn format_loop_options(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: u_int = 0 as u_int;
    let mut n: u_int = 0;
    let mut global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags.is_null() || *flags as ::core::ffi::c_int == '\0' as i32 {
        flags = b"s\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !strchr(flags, 'v' as i32).is_null() {
        oo = global_options;
    } else {
        if !strchr(flags, 'g' as i32).is_null() {
            global = 1 as ::core::ffi::c_int;
        }
        if !strchr(flags, 'w' as i32).is_null() {
            if global != 0 {
                oo = global_w_options;
            } else if !(*ft).w.is_null() {
                oo = (*(*ft).w).options;
            }
        } else if !strchr(flags, 's' as i32).is_null() {
            if global != 0 {
                oo = global_s_options;
            } else if !(*ft).s.is_null() {
                oo = (*(*ft).s).options;
            }
        } else if !strchr(flags, 'p' as i32).is_null() {
            if !(global != 0) {
                if !(*ft).wp.is_null() {
                    oo = (*(*ft).wp).options;
                }
            }
        } else if global != 0 {
            oo = global_s_options;
        }
    }
    if oo.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    o = options_first(oo);
    while !o.is_null() {
        n = 0 as u_int;
        if options_is_array(o) != 0 {
            a = options_array_first(o);
            while !a.is_null() {
                n = n.wrapping_add(1);
                a = options_array_next(a);
            }
        }
        if options_is_array(o) == 0 || n == 0 as u_int {
            format_loop_add_option(es, fmt, buffer, o, n, i);
            i = i.wrapping_add(1);
            o = options_next(o);
        } else {
            a = options_array_first(o);
            while !a.is_null() {
                format_loop_add_array_item(es, fmt, buffer, o, a, n as ::core::ffi::c_int, i);
                i = i.wrapping_add(1);
                a = options_array_next(a);
            }
            o = options_next(o);
        }
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_loop_environ(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: u_int = 0 as u_int;
    if flags.is_null()
        || *flags as ::core::ffi::c_int == '\0' as i32
        || strcmp(flags, b"s\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        if !(*ft).s.is_null() {
            env = (*(*ft).s).environ;
        }
    } else if strcmp(flags, b"g\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        env = global_environ;
    } else if strcmp(flags, b"c\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        if !(*ft).client.is_null() {
            env = (*(*ft).client).environ;
        }
    }
    if env.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    envent = environ_first(env);
    while !envent.is_null() {
        format_log1(
            es,
            b"format_loop_environ\0" as *const u8 as *const ::core::ffi::c_char,
            b"environment loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
        nft = format_create(c, item, FORMAT_NONE, (*ft).flags);
        format_add(
            nft,
            b"environ_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
        if (*envent).value.is_null() {
            format_add(
                nft,
                b"environ_value\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"environ_value\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).value,
            );
        }
        if (*envent).flags & ENVIRON_HIDDEN != 0 {
            format_add(
                nft,
                b"environ_hidden\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"environ_hidden\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_add(
            nft,
            b"environ_removed\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).value == NULL_0 as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
        );
        if environ_next(envent).is_null() {
            format_add(
                nft,
                b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, fmt);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
        envent = environ_next(envent);
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_loop_clients(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    let clients_sorted = sort_get_clients(sc);
    let n = ::core::ffi::c_int::try_from(clients_sorted.len())
        .expect("too many clients for format loop");
    i = 0 as ::core::ffi::c_int;
    while i < n {
        c = clients_sorted[i as usize];
        format_log1(
            es,
            b"format_loop_clients\0" as *const u8 as *const ::core::ffi::c_char,
            b"client loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        nft = format_create(c, item, 0 as ::core::ffi::c_int, (*ft).flags);
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(nft, c, (*ft).s, (*ft).wl, (*ft).wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, fmt);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
pub(super) unsafe extern "C" fn format_replace_expression(
    mut mexp: *mut format_modifier,
    mut es: *mut format_expand_state,
    mut copy: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut argc: ::core::ffi::c_int = (*mexp).argc();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut endch: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut left: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut right: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_fp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prec: u_int = 0 as u_int;
    let mut mleft: ::core::ffi::c_double = 0.;
    let mut mright: ::core::ffi::c_double = 0.;
    let mut result: ::core::ffi::c_double = 0.;
    let mut operator: C2RustUnnamed_44 = ADD;
    if strcmp(
        (*mexp).arg(0),
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = ADD;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = SUBTRACT;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"*\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = MULTIPLY;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"/\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = DIVIDE;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"%\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            (*mexp).arg(0),
            b"m\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        operator = MODULUS;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"==\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"!=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = NOT_EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b">\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = GREATER_THAN;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"<\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = LESS_THAN;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b">=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = GREATER_THAN_EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        (*mexp).arg(0),
        b"<=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = LESS_THAN_EQUAL;
        current_block = 4495394744059808450;
    } else {
        format_log1(
            es,
            b"format_replace_expression\0" as *const u8 as *const ::core::ffi::c_char,
            b"expression has no valid operator: '%s'\0" as *const u8 as *const ::core::ffi::c_char,
            (*mexp).arg(0),
        );
        current_block = 7376217411786091060;
    }
    match current_block {
        4495394744059808450 => {
            if argc >= 2 as ::core::ffi::c_int && !strchr((*mexp).arg(1), 'f' as i32).is_null() {
                use_fp = 1 as ::core::ffi::c_int;
                prec = 2 as u_int;
            }
            if argc >= 3 as ::core::ffi::c_int {
                prec = strtonum(
                    (*mexp).arg(2),
                    -FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                    FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    format_log1(
                        es,
                        b"format_replace_expression\0" as *const u8 as *const ::core::ffi::c_char,
                        b"expression precision %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        errstr,
                        (*mexp).arg(2),
                    );
                    current_block = 7376217411786091060;
                } else {
                    current_block = 3437258052017859086;
                }
            } else {
                current_block = 3437258052017859086;
            }
            match current_block {
                7376217411786091060 => {}
                _ => {
                    if format_choose(es, copy, &raw mut left, &raw mut right)
                        != 0 as ::core::ffi::c_int
                    {
                        format_log1(
                            es,
                            b"format_replace_expression\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"expression syntax error\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        mleft = strtod(left, &raw mut endch);
                        if *endch as ::core::ffi::c_int != '\0' as i32 {
                            format_log1(
                                es,
                                b"format_replace_expression\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"expression left side is invalid: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                left,
                            );
                        } else {
                            mright = strtod(right, &raw mut endch);
                            if *endch as ::core::ffi::c_int != '\0' as i32 {
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression right side is invalid: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    right,
                                );
                            } else {
                                if use_fp == 0 {
                                    mleft =
                                        mleft as ::core::ffi::c_longlong as ::core::ffi::c_double;
                                    mright =
                                        mright as ::core::ffi::c_longlong as ::core::ffi::c_double;
                                }
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression left side is: %.*f\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    prec,
                                    mleft,
                                );
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression right side is: %.*f\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    prec,
                                    mright,
                                );
                                match operator as ::core::ffi::c_uint {
                                    0 => {
                                        result = mleft + mright;
                                    }
                                    1 => {
                                        result = mleft - mright;
                                    }
                                    2 => {
                                        result = mleft * mright;
                                    }
                                    3 => {
                                        result = mleft / mright;
                                    }
                                    4 => {
                                        result = fmod(mleft, mright);
                                    }
                                    5 => {
                                        result = (fabs(mleft - mright) < 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    6 => {
                                        result = (fabs(mleft - mright) > 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    7 => {
                                        result = (mleft > mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    8 => {
                                        result = (mleft >= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    9 => {
                                        result = (mleft < mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    10 => {
                                        result = (mleft <= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    _ => {}
                                }
                                if use_fp != 0 {
                                    xasprintf(
                                        &raw mut value,
                                        b"%.*f\0" as *const u8 as *const ::core::ffi::c_char,
                                        prec,
                                        result,
                                    );
                                } else {
                                    xasprintf(
                                        &raw mut value,
                                        b"%.*f\0" as *const u8 as *const ::core::ffi::c_char,
                                        prec,
                                        result as ::core::ffi::c_longlong as ::core::ffi::c_double,
                                    );
                                }
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression result is %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    value,
                                );
                                free(right as *mut ::core::ffi::c_void);
                                free(left as *mut ::core::ffi::c_void);
                                return value;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    free(right as *mut ::core::ffi::c_void);
    free(left as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub(super) unsafe extern "C" fn format_cycle_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    if (*c).message_string.is_null() && (*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
}
pub(super) unsafe extern "C" fn format_cycle_start_timer(mut c: *mut client) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    tv.tv_sec = (FORMAT_CYCLE_PERIOD / 1000 as ::core::ffi::c_int) as __time_t;
    tv.tv_usec = ((FORMAT_CYCLE_PERIOD % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    if event_initialized(&raw mut (*c).cycle_timer) == 0 {
        event_set(
            &raw mut (*c).cycle_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                format_cycle_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*c).cycle_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*c).cycle_timer, &raw mut tv);
    }
}
pub(super) unsafe extern "C" fn format_cycle(
    mut es: *mut format_expand_state,
    mut frames: *const ::core::ffi::c_char,
    mut count: u_int,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut index: u_int = 0;
    let mut i: u_int = 0;
    if (*ft).flags & FORMAT_STATUS == 0 || (*es).flags & FORMAT_EXPAND_NOCYCLE != 0 {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if *frames as ::core::ffi::c_int == '\0' as i32 {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    n = 1 as u_int;
    cp = frames;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == ',' as i32 {
            n = n.wrapping_add(1);
        }
        cp = cp.offset(1);
    }
    index = (*es)
        .start_time
        .wrapping_div(count.wrapping_mul(FORMAT_CYCLE_PERIOD as u_int) as uint64_t)
        .wrapping_rem(n as uint64_t) as u_int;
    if n > 1 as u_int && !(*ft).client.is_null() {
        format_cycle_start_timer((*ft).client);
    }
    start = frames;
    i = 0 as u_int;
    while i < index {
        start = strchr(start, ',' as i32).offset(1 as ::core::ffi::c_int as isize);
        i = i.wrapping_add(1);
    }
    end = strchr(start, ',' as i32);
    if end.is_null() {
        end = start.offset(strlen(start) as isize);
    }
    return xstrndup(
        start,
        end.offset_from(start) as ::core::ffi::c_long as size_t,
    );
}
pub(super) unsafe extern "C" fn format_replace(
    mut es: *mut format_expand_state,
    mut key: *const ::core::ffi::c_char,
    mut keylen: size_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut off: *mut size_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut wp: *mut window_pane = (*ft).wp;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut copy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut marker: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut time_format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut left: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut right: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut valuelen: size_t = 0;
    let mut modifiers: uint64_t = 0 as uint64_t;
    let mut limit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut list: Vec<format_modifier> = Vec::new();
    let mut cmp: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut search: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut sub: Vec<*mut format_modifier> = Vec::new();
    let mut mexp: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut fm: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut bool_op_n: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut cycle_count: u_int = 1 as u_int;
    let mut i: u_int = 0;
    let mut nrep: u_int = 0;
    let mut check: u_int = 0 as u_int;
    let mut loop_flags: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    (*sc).order = SORT_ORDER;
    (*sc).reversed = 0 as ::core::ffi::c_int;
    // Match strndup's bounded scan, including an early NUL.
    let key_end = libc::strnlen(key, keylen);
    let copy0 = CString::new(std::slice::from_raw_parts(key.cast::<u8>(), key_end))
        .expect("format key contains no NUL");
    copy = copy0.as_ptr();
    list = format_build_modifiers(es, &raw mut copy);
    // No more entries are pushed after parsing. Pointers saved in cmp, search,
    // sub, and other modifier selections stay valid until cleanup.
    i = 0 as u_int;
    while (i as usize) < list.len() {
        fm = list.as_mut_ptr().add(i as usize);
        if format_logging(ft) != 0 {
            format_log1(
                es,
                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                b"modifier %u is %s\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
            );
            j = 0 as ::core::ffi::c_int;
            while j < (*fm).argc() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"modifier %u argument %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    i,
                    j,
                    (*fm).arg(j as usize),
                );
                j += 1;
            }
        }
        if (*fm).size == 1 as u_int {
            match (*fm).modifier[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
                109 | 60 | 62 => {
                    cmp = fm;
                }
                33 => {
                    modifiers |= FORMAT_NOT as uint64_t;
                }
                67 => {
                    search = fm;
                }
                115 => {
                    if !((*fm).argc() < 2 as ::core::ffi::c_int) {
                        sub.push(fm);
                    }
                }
                61 => {
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        limit = strtonum(
                            (*fm).arg(0),
                            -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as ::core::ffi::c_int;
                        if !errstr.is_null() {
                            limit = 0 as ::core::ffi::c_int;
                        }
                        if (*fm).argc() >= 2 as ::core::ffi::c_int {
                            marker = (*fm).arg(1);
                        }
                    }
                }
                112 => {
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        width = strtonum(
                            (*fm).arg(0),
                            -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as ::core::ffi::c_int;
                        if !errstr.is_null() {
                            width = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                65 => {
                    modifiers = (modifiers as ::core::ffi::c_ulonglong | FORMAT_CYCLE) as uint64_t;
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        cycle_count = strtonum(
                            (*fm).arg(0),
                            1 as ::core::ffi::c_longlong,
                            100 as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as u_int;
                        if !errstr.is_null() {
                            cycle_count = 1 as u_int;
                        }
                    }
                }
                119 => {
                    modifiers |= FORMAT_WIDTH as uint64_t;
                }
                101 => {
                    if !((*fm).argc() < 1 as ::core::ffi::c_int
                        || (*fm).argc() > 3 as ::core::ffi::c_int)
                    {
                        mexp = fm;
                    }
                }
                108 => {
                    modifiers |= FORMAT_LITERAL as uint64_t;
                }
                97 => {
                    modifiers |= FORMAT_CHARACTER as uint64_t;
                }
                98 => {
                    modifiers |= FORMAT_BASENAME as uint64_t;
                }
                99 => {
                    modifiers |= FORMAT_COLOUR as uint64_t;
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        if !strchr((*fm).arg(0), 'f' as i32).is_null() {
                            modifiers |= FORMAT_COLOUR_ESC_FG as uint64_t;
                        }
                        if !strchr((*fm).arg(0), 'b' as i32).is_null() {
                            modifiers |= FORMAT_COLOUR_ESC_BG as uint64_t;
                        }
                    }
                }
                100 => {
                    modifiers |= FORMAT_DIRNAME as uint64_t;
                }
                110 => {
                    modifiers |= FORMAT_LENGTH as uint64_t;
                }
                73 => {
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        if !strchr((*fm).arg(0), 'f' as i32).is_null() {
                            modifiers |= FORMAT_CLIENT_TERMFEAT as uint64_t;
                        }
                        if !strchr((*fm).arg(0), 'c' as i32).is_null() {
                            modifiers |= FORMAT_CLIENT_TERMCAP as uint64_t;
                        }
                        if !strchr((*fm).arg(0), 'e' as i32).is_null() {
                            modifiers |= FORMAT_CLIENT_ENVIRON as uint64_t;
                        }
                    }
                }
                116 => {
                    modifiers |= FORMAT_TIMESTRING as uint64_t;
                    if !((*fm).argc() < 1 as ::core::ffi::c_int) {
                        if !strchr((*fm).arg(0), 'p' as i32).is_null() {
                            modifiers |= FORMAT_PRETTY as uint64_t;
                        } else if !strchr((*fm).arg(0), 'r' as i32).is_null() {
                            modifiers |= FORMAT_RELATIVE as uint64_t;
                        } else if !strchr((*fm).arg(0), 'd' as i32).is_null() {
                            modifiers = (modifiers as ::core::ffi::c_ulonglong | FORMAT_DIFFERENCE)
                                as uint64_t;
                        } else if (*fm).argc() >= 2 as ::core::ffi::c_int
                            && !strchr((*fm).arg(0), 'f' as i32).is_null()
                        {
                            free(time_format as *mut ::core::ffi::c_void);
                            time_format = format_strip(es, (*fm).arg(1));
                        }
                    }
                }
                113 => {
                    if (*fm).argc() < 1 as ::core::ffi::c_int {
                        modifiers |= FORMAT_QUOTE_SHELL as uint64_t;
                    } else if !strchr((*fm).arg(0), 's' as i32).is_null() {
                        modifiers |= FORMAT_QUOTE_SHELL_SQ as uint64_t;
                    } else if !strchr((*fm).arg(0), 'e' as i32).is_null()
                        || !strchr((*fm).arg(0), 'h' as i32).is_null()
                    {
                        modifiers |= FORMAT_QUOTE_STYLE as uint64_t;
                    } else if !strchr((*fm).arg(0), 'a' as i32).is_null() {
                        modifiers |= FORMAT_QUOTE_ARGUMENTS as uint64_t;
                    }
                }
                69 => {
                    modifiers |= FORMAT_EXPAND as uint64_t;
                }
                84 => {
                    modifiers |= FORMAT_EXPANDTIME as uint64_t;
                }
                78 => {
                    if (*fm).argc() < 1 as ::core::ffi::c_int
                        || !strchr((*fm).arg(0), 'w' as i32).is_null()
                    {
                        modifiers |= FORMAT_WINDOW_NAME as uint64_t;
                    } else if !strchr((*fm).arg(0), 's' as i32).is_null() {
                        modifiers |= FORMAT_SESSION_NAME as uint64_t;
                    }
                }
                83 => {
                    modifiers |= FORMAT_SESSIONS as uint64_t;
                    if (*fm).argc() < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_INDEX;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
                            (*sc).order = SORT_INDEX;
                        } else if !strchr((*fm).arg(0), 'n' as i32).is_null() {
                            (*sc).order = SORT_NAME;
                        } else if !strchr((*fm).arg(0), 't' as i32).is_null() {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_INDEX;
                        }
                        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                87 => {
                    modifiers |= FORMAT_WINDOWS as uint64_t;
                    if (*fm).argc() < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_ORDER;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
                            (*sc).order = SORT_ORDER;
                        } else if !strchr((*fm).arg(0), 'n' as i32).is_null() {
                            (*sc).order = SORT_NAME;
                        } else if !strchr((*fm).arg(0), 't' as i32).is_null() {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_ORDER;
                        }
                        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                80 => {
                    modifiers |= FORMAT_PANES as uint64_t;
                    (*sc).order = SORT_CREATION;
                    if (*fm).argc() < 1 as ::core::ffi::c_int {
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
                            (*sc).order = SORT_INDEX;
                        } else if !strchr((*fm).arg(0), 'z' as i32).is_null() {
                            (*sc).order = SORT_Z;
                        } else {
                            (*sc).order = SORT_CREATION;
                        }
                        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                79 => {
                    modifiers |= FORMAT_OPTIONS as uint64_t;
                    if (*fm).argc() == 1 as ::core::ffi::c_int {
                        loop_flags = (*fm).arg(0);
                    }
                }
                86 => {
                    modifiers =
                        (modifiers as ::core::ffi::c_ulonglong | FORMAT_ENVIRON) as uint64_t;
                    if (*fm).argc() == 1 as ::core::ffi::c_int {
                        loop_flags = (*fm).arg(0);
                    }
                }
                76 => {
                    modifiers |= FORMAT_CLIENTS as uint64_t;
                    if (*fm).argc() < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_ORDER;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
                            (*sc).order = SORT_ORDER;
                        } else if !strchr((*fm).arg(0), 'n' as i32).is_null() {
                            (*sc).order = SORT_NAME;
                        } else if !strchr((*fm).arg(0), 't' as i32).is_null() {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_ORDER;
                        }
                        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                82 => {
                    modifiers |= FORMAT_REPEAT as uint64_t;
                }
                _ => {}
            }
        } else if (*fm).size == 2 as u_int {
            if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"||\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"&&\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                bool_op_n = fm;
            } else if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"!!\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                modifiers |= FORMAT_NOT_NOT as uint64_t;
            } else if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"==\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"!=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b">=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"<=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                cmp = fm;
            }
        }
        i = i.wrapping_add(1);
    }
    if modifiers & FORMAT_CLIENT_TERMCAP as uint64_t != 0
        || modifiers & FORMAT_CLIENT_TERMFEAT as uint64_t != 0
        || modifiers & FORMAT_CLIENT_ENVIRON as uint64_t != 0
    {
        if (*ft).c.is_null()
            || (*(*ft).c).tty.term.is_null()
            || (*(*ft).c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0
        {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            if modifiers & FORMAT_CLIENT_TERMCAP as uint64_t != 0 {
                if tty_term_has_name((*(*ft).c).tty.term, copy) != 0 {
                    value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            if modifiers & FORMAT_CLIENT_TERMFEAT as uint64_t != 0 {
                if tty_feature_present((*(*ft).c).tty.term, copy) != 0 {
                    value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            if modifiers & FORMAT_CLIENT_ENVIRON as uint64_t != 0 {
                envent = environ_find((*(*ft).c).environ, copy);
                if !envent.is_null() && !(*envent).value.is_null() {
                    value = xstrdup((*envent).value);
                } else {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
        }
    } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_CYCLE != 0 {
        value = format_cycle(es, copy, cycle_count);
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"cycle '%s' is: %s\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
            value,
        );
    } else if modifiers & FORMAT_LITERAL as uint64_t != 0 {
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"literal string is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
        );
        value = format_unescape(es, copy, strlen(copy));
    } else if modifiers & FORMAT_CHARACTER as uint64_t != 0 {
        new = format_expand1(es, copy);
        c = strtonum(
            new,
            32 as ::core::ffi::c_longlong,
            126 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            xasprintf(
                &raw mut value,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                c,
            );
        }
        free(new as *mut ::core::ffi::c_void);
    } else if modifiers & FORMAT_COLOUR as uint64_t != 0 {
        new = format_expand1(es, copy);
        if modifiers & (FORMAT_COLOUR_ESC_FG | FORMAT_COLOUR_ESC_BG) as uint64_t != 0 {
            if strcasecmp(new, b"none\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                value = xstrdup(b"\x1B[0m\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                c = colour_parse_cstr(std::ffi::CStr::from_ptr(new)).unwrap_or(-1);
                if c == -(1 as ::core::ffi::c_int) {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    let escape = colour_format_escape_for_client(
                        (*ft).c,
                        c,
                        modifiers & FORMAT_COLOUR_ESC_BG as uint64_t != 0,
                    );
                    value = xstrdup(escape.as_deref().unwrap_or(c"").as_ptr());
                }
            }
        } else {
            c = colour_parse_cstr(std::ffi::CStr::from_ptr(new)).unwrap_or(-1);
            if c == -(1 as ::core::ffi::c_int) || {
                c = colour_force_rgb(c);
                c == -(1 as ::core::ffi::c_int)
            } {
                value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                xasprintf(
                    &raw mut value,
                    b"%06x\0" as *const u8 as *const ::core::ffi::c_char,
                    c & 0xffffff as ::core::ffi::c_int,
                );
            }
        }
        free(new as *mut ::core::ffi::c_void);
    } else {
        if modifiers & FORMAT_SESSIONS as uint64_t != 0 {
            value = format_loop_sessions(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_WINDOWS as uint64_t != 0 {
            value = format_loop_windows(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_PANES as uint64_t != 0 {
            value = format_loop_panes(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_CLIENTS as uint64_t != 0 {
            value = format_loop_clients(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_OPTIONS as uint64_t != 0 {
            value = format_loop_options(es, copy, loop_flags);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_ENVIRON != 0 {
            value = format_loop_environ(es, copy, loop_flags);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_WINDOW_NAME as uint64_t != 0 {
            value = format_window_name(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_SESSION_NAME as uint64_t != 0 {
            value = format_session_name(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if !search.is_null() {
            new = format_expand1(es, copy);
            if wp.is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' but no pane\0" as *const u8 as *const ::core::ffi::c_char,
                    new,
                );
                value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    new,
                    (*wp).id,
                );
                value = format_search(search, wp, new);
            }
            free(new as *mut ::core::ffi::c_void);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_REPEAT as uint64_t != 0 {
            if format_choose(es, copy, &raw mut left, &raw mut right) != 0 as ::core::ffi::c_int {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"repeat syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                nrep = strtonum(
                    right,
                    1 as ::core::ffi::c_longlong,
                    FORMAT_MAX_REPEAT as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    current_block = 6055351187523413397;
                } else {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    i = 0 as u_int;
                    loop {
                        if !(i < nrep) {
                            current_block = 6055351187523413397;
                            break;
                        }
                        if format_check_time(es, &raw mut check) == 0 {
                            free(right as *mut ::core::ffi::c_void);
                            free(left as *mut ::core::ffi::c_void);
                            free(value as *mut ::core::ffi::c_void);
                            current_block = 6506207624831006569;
                            break;
                        } else {
                            xasprintf(
                                &raw mut new,
                                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                value,
                                left,
                            );
                            free(value as *mut ::core::ffi::c_void);
                            value = new;
                            i = i.wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    6506207624831006569 => {}
                    _ => {
                        free(right as *mut ::core::ffi::c_void);
                        free(left as *mut ::core::ffi::c_void);
                        current_block = 1803726662341650892;
                    }
                }
            }
        } else if modifiers & FORMAT_NOT as uint64_t != 0 {
            value = format_bool_op_1(es, copy, 1 as ::core::ffi::c_int);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_NOT_NOT as uint64_t != 0 {
            value = format_bool_op_1(es, copy, 0 as ::core::ffi::c_int);
            current_block = 1803726662341650892;
        } else if !bool_op_n.is_null() {
            if strcmp(
                &raw mut (*bool_op_n).modifier as *mut ::core::ffi::c_char,
                b"||\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                value = format_bool_op_n(es, copy, 0 as ::core::ffi::c_int);
            } else if strcmp(
                &raw mut (*bool_op_n).modifier as *mut ::core::ffi::c_char,
                b"&&\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                value = format_bool_op_n(es, copy, 1 as ::core::ffi::c_int);
            }
            current_block = 1803726662341650892;
        } else if !cmp.is_null() {
            if format_choose(es, copy, &raw mut left, &raw mut right) != 0 as ::core::ffi::c_int {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s left is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    left,
                );
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s right is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    right,
                );
                if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"==\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) == 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"!=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) != 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) < 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) > 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) <= 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) >= 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"m\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    value = format_match(cmp, left, right);
                }
                free(right as *mut ::core::ffi::c_void);
                free(left as *mut ::core::ffi::c_void);
                current_block = 1803726662341650892;
            }
        } else {
            if *copy as ::core::ffi::c_int == '?' as i32 {
                cp = copy.offset(1 as ::core::ffi::c_int as isize);
                loop {
                    cp2 = format_skip1(es, cp, b",\0" as *const u8 as *const ::core::ffi::c_char);
                    if cp2.is_null() {
                        format_log1(
                            es,
                            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                            b"no condition matched in '%s'; using last arg\0" as *const u8
                                as *const ::core::ffi::c_char,
                            copy.offset(1 as ::core::ffi::c_int as isize),
                        );
                        value = format_expand1(es, cp);
                        break;
                    } else {
                        // The delimiter is inside the NUL-terminated input, so
                        // this slice contains only the condition's bytes.
                        let condition = CString::new(std::slice::from_raw_parts(
                            cp.cast::<u8>(),
                            cp2.offset_from(cp) as usize,
                        ))
                        .expect("format condition contains no NUL");
                        format_log1(
                            es,
                            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                            b"condition is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            condition.as_ptr(),
                        );
                        found = format_find(ft, condition.as_ptr(), modifiers, time_format);
                        if found.is_null() {
                            found = format_expand1(es, condition.as_ptr());
                            if strcmp(found, condition.as_ptr()) == 0 as ::core::ffi::c_int {
                                free(found as *mut ::core::ffi::c_void);
                                found = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"condition '%s' not found; assuming false\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    condition.as_ptr(),
                                );
                            }
                        } else {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' found: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition.as_ptr(),
                                found,
                            );
                        }
                        cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                        cp2 =
                            format_skip1(es, cp, b",\0" as *const u8 as *const ::core::ffi::c_char);
                        if format_true(found) != 0 {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' is true\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition.as_ptr(),
                            );
                            if cp2.is_null() {
                                value = format_expand1(es, cp);
                            } else {
                                let right = CString::new(std::slice::from_raw_parts(
                                    cp.cast::<u8>(),
                                    cp2.offset_from(cp) as usize,
                                ))
                                .expect("format branch contains no NUL");
                                value = format_expand1(es, right.as_ptr());
                                drop(right);
                            }
                            drop(condition);
                            free(found as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' is false\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition.as_ptr(),
                            );
                            drop(condition);
                            free(found as *mut ::core::ffi::c_void);
                            if cp2.is_null() {
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"no condition matched in '%s'; using empty string\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                    copy.offset(1 as ::core::ffi::c_int as isize),
                                );
                                value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                                break;
                            } else {
                                cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                            }
                        }
                    }
                }
            } else if !mexp.is_null() {
                value = format_replace_expression(mexp, es, copy);
                if value.is_null() {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                }
            } else if !strstr(copy, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"expanding inner format '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                value = format_expand1(es, copy);
            } else {
                value = format_find(ft, copy, modifiers, time_format);
                if value.is_null() {
                    format_log1(
                        es,
                        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format '%s' not found\0" as *const u8 as *const ::core::ffi::c_char,
                        copy,
                    );
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    format_log1(
                        es,
                        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format '%s' found: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        copy,
                        value,
                    );
                }
            }
            current_block = 1803726662341650892;
        }
        match current_block {
            1803726662341650892 => {}
            _ => {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"failed %s\0" as *const u8 as *const ::core::ffi::c_char,
                    copy0.as_ptr(),
                );
                drop(sub);
                drop(list);
                drop(copy0);
                free(time_format as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
        }
    }
    if modifiers & FORMAT_EXPAND as uint64_t != 0 {
        new = format_expand1(es, value);
        free(value as *mut ::core::ffi::c_void);
        value = new;
    } else if modifiers & FORMAT_EXPANDTIME as uint64_t != 0 {
        format_copy_state(&raw mut next, es, FORMAT_EXPAND_TIME);
        new = format_expand1(&raw mut next, value);
        free(value as *mut ::core::ffi::c_void);
        value = new;
    }
    i = 0 as u_int;
    while (i as usize) < sub.len() {
        let modifier = sub[i as usize];
        left = format_expand1(es, (*modifier).arg(0));
        right = format_expand1(es, (*modifier).arg(1));
        new = format_sub(modifier, value, left, right);
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"substitute '%s' to '%s': %s\0" as *const u8 as *const ::core::ffi::c_char,
            left,
            right,
            new,
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        free(right as *mut ::core::ffi::c_void);
        free(left as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    if limit > 0 as ::core::ffi::c_int {
        new = format_trim_left(value, limit as u_int);
        if !marker.is_null() && strcmp(new, value) != 0 as ::core::ffi::c_int {
            free(value as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut value,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                new,
                marker,
            );
            free(new as *mut ::core::ffi::c_void);
        } else {
            free(value as *mut ::core::ffi::c_void);
            value = new;
        }
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied length limit %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            limit,
            value,
        );
    } else if limit < 0 as ::core::ffi::c_int {
        new = format_trim_right(value, -limit as u_int);
        if !marker.is_null() && strcmp(new, value) != 0 as ::core::ffi::c_int {
            free(value as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut value,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                marker,
                new,
            );
            free(new as *mut ::core::ffi::c_void);
        } else {
            free(value as *mut ::core::ffi::c_void);
            value = new;
        }
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied length limit %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            limit,
            value,
        );
    }
    if width > 0 as ::core::ffi::c_int {
        new = utf8_padcstr(value, width as u_int);
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied padding width %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            width,
            value,
        );
    } else if width < 0 as ::core::ffi::c_int {
        new = utf8_rpadcstr(value, -width as u_int);
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied padding width %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            width,
            value,
        );
    }
    if modifiers & FORMAT_LENGTH as uint64_t != 0 {
        xasprintf(
            &raw mut new,
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            strlen(value),
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"replacing with length: %s\0" as *const u8 as *const ::core::ffi::c_char,
            new,
        );
    }
    if modifiers & FORMAT_WIDTH as uint64_t != 0 {
        xasprintf(
            &raw mut new,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            format_width(value),
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"replacing with width: %s\0" as *const u8 as *const ::core::ffi::c_char,
            new,
        );
    }
    valuelen = strlen(value);
    while (*len).wrapping_sub(*off) < valuelen.wrapping_add(1 as size_t) {
        *buf = xreallocarray(*buf as *mut ::core::ffi::c_void, 2 as size_t, *len)
            as *mut ::core::ffi::c_char;
        *len = (*len).wrapping_mul(2 as size_t);
    }
    memcpy(
        (*buf).offset(*off as isize) as *mut ::core::ffi::c_void,
        value as *const ::core::ffi::c_void,
        valuelen,
    );
    *off = (*off).wrapping_add(valuelen);
    format_log1(
        es,
        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
        b"replaced '%s' with '%s'\0" as *const u8 as *const ::core::ffi::c_char,
        copy0.as_ptr(),
        value,
    );
    free(value as *mut ::core::ffi::c_void);
    drop(sub);
    drop(list);
    drop(copy0);
    free(time_format as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
pub(super) unsafe extern "C" fn format_expand1(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style_end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut off: size_t = 0;
    let mut len: size_t = 0;
    let mut n: size_t = 0;
    let mut outlen: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 0;
    let mut expanded: [::core::ffi::c_char; 8192] = [0; 8192];
    if fmt.is_null()
        || *fmt as ::core::ffi::c_int == '\0' as i32
        || format_check_time(es, ::core::ptr::null_mut::<u_int>()) == 0
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*es).loop_0 == FORMAT_LOOP_LIMIT as u_int {
        format_log1(
            es,
            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
            b"reached loop limit (%u)\0" as *const u8 as *const ::core::ffi::c_char,
            FORMAT_LOOP_LIMIT,
        );
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*es).loop_0 = (*es).loop_0.wrapping_add(1);
    format_log1(
        es,
        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
        b"expanding format: %s\0" as *const u8 as *const ::core::ffi::c_char,
        fmt,
    );
    if (*es).flags & FORMAT_EXPAND_TIME != 0 && !strchr(fmt, '%' as i32).is_null() {
        if (*es).time == 0 as time_t {
            (*es).time = time(::core::ptr::null_mut::<time_t>());
            localtime_r(&raw mut (*es).time, &raw mut (*es).tm);
        }
        if format_strftime(
            &raw mut expanded as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
            fmt,
            &raw mut (*es).tm,
        ) == 0 as size_t
        {
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"format is too long\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if format_logging(ft) != 0
            && strcmp(&raw mut expanded as *mut ::core::ffi::c_char, fmt) != 0 as ::core::ffi::c_int
        {
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"after time expanded: %s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut expanded as *mut ::core::ffi::c_char,
            );
        }
        fmt = &raw mut expanded as *mut ::core::ffi::c_char;
    }
    len = 64 as size_t;
    buf = xmalloc(len) as *mut ::core::ffi::c_char;
    off = 0 as size_t;
    while *fmt as ::core::ffi::c_int != '\0' as i32 {
        if *fmt as ::core::ffi::c_int != '#' as i32 {
            while len.wrapping_sub(off) < 2 as size_t {
                buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                    as *mut ::core::ffi::c_char;
                len = len.wrapping_mul(2 as size_t);
            }
            let fresh10 = fmt;
            fmt = fmt.offset(1);
            let fresh11 = off;
            off = off.wrapping_add(1);
            *buf.offset(fresh11 as isize) = *fresh10;
        } else {
            fmt = fmt.offset(1);
            if *fmt as ::core::ffi::c_int == '\0' as i32 {
                break;
            }
            let fresh12 = fmt;
            fmt = fmt.offset(1);
            ch = *fresh12 as u_char as ::core::ffi::c_int;
            match ch {
                40 => {
                    brackets = 1 as ::core::ffi::c_int;
                    ptr = fmt;
                    while *ptr as ::core::ffi::c_int != '\0' as i32 {
                        if *ptr as ::core::ffi::c_int == '(' as i32 {
                            brackets += 1;
                        }
                        if *ptr as ::core::ffi::c_int == ')' as i32 && {
                            brackets -= 1;
                            brackets == 0 as ::core::ffi::c_int
                        } {
                            break;
                        }
                        ptr = ptr.offset(1);
                    }
                    if *ptr as ::core::ffi::c_int != ')' as i32
                        || brackets != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    n = ptr.offset_from(fmt) as ::core::ffi::c_long as size_t;
                    // The closing ')' is before the terminating NUL in fmt.
                    let name = CString::new(std::slice::from_raw_parts(fmt.cast::<u8>(), n))
                        .expect("format job name contains no NUL");
                    format_log1(
                        es,
                        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                        b"found #(): %s\0" as *const u8 as *const ::core::ffi::c_char,
                        name.as_ptr(),
                    );
                    if (*ft).flags & FORMAT_NOJOBS != 0 || (*es).flags & FORMAT_EXPAND_NOJOBS != 0 {
                        out = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() is disabled\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        out = format_job_get(es, name.as_ptr());
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() result: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            out,
                        );
                    }
                    outlen = strlen(out);
                    while len.wrapping_sub(off) < outlen.wrapping_add(1 as size_t) {
                        buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                            as *mut ::core::ffi::c_char;
                        len = len.wrapping_mul(2 as size_t);
                    }
                    memcpy(
                        buf.offset(off as isize) as *mut ::core::ffi::c_void,
                        out as *const ::core::ffi::c_void,
                        outlen,
                    );
                    off = off.wrapping_add(outlen);
                    free(out as *mut ::core::ffi::c_void);
                    fmt = fmt.offset(n.wrapping_add(1 as size_t) as isize);
                    continue;
                }
                123 => {
                    ptr = format_skip1(
                        es,
                        (fmt as *mut ::core::ffi::c_char)
                            .offset(-(2 as ::core::ffi::c_int as isize)),
                        b"}\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if ptr.is_null() {
                        break;
                    }
                    n = ptr.offset_from(fmt) as ::core::ffi::c_long as size_t;
                    format_log1(
                        es,
                        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                        b"found #{}: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                        n as ::core::ffi::c_int,
                        fmt,
                    );
                    if format_replace(es, fmt, n, &raw mut buf, &raw mut len, &raw mut off)
                        != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    fmt = fmt.offset(n.wrapping_add(1 as size_t) as isize);
                    continue;
                }
                91 | 35 => {
                    ptr = fmt.offset(-((ch == '[' as i32) as ::core::ffi::c_int as isize));
                    n = (2 as ::core::ffi::c_int - (ch == '[' as i32) as ::core::ffi::c_int)
                        as size_t;
                    while *ptr as ::core::ffi::c_int == '#' as i32 {
                        ptr = ptr.offset(1);
                        n = n.wrapping_add(1);
                    }
                    if *ptr as ::core::ffi::c_int == '[' as i32 {
                        style_end = format_skip1(
                            es,
                            fmt.offset(-(2 as ::core::ffi::c_int as isize)),
                            b"]\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"found #*%zu[\0" as *const u8 as *const ::core::ffi::c_char,
                            n,
                        );
                        while len.wrapping_sub(off) < n.wrapping_add(2 as size_t) {
                            buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                                as *mut ::core::ffi::c_char;
                            len = len.wrapping_mul(2 as size_t);
                        }
                        memcpy(
                            buf.offset(off as isize) as *mut ::core::ffi::c_void,
                            fmt.offset(-(2 as ::core::ffi::c_int as isize))
                                as *const ::core::ffi::c_void,
                            n.wrapping_add(1 as size_t),
                        );
                        off = off.wrapping_add(n.wrapping_add(1 as size_t));
                        fmt = ptr.offset(1 as ::core::ffi::c_int as isize);
                        continue;
                    }
                }
                125 | 44 => {}
                _ => {
                    s = ::core::ptr::null::<::core::ffi::c_char>();
                    if fmt > style_end {
                        if ch >= 'A' as i32 && ch <= 'Z' as i32 {
                            s = format_upper[(ch - 'A' as i32) as usize];
                        } else if ch >= 'a' as i32 && ch <= 'z' as i32 {
                            s = format_lower[(ch - 'a' as i32) as usize];
                        }
                    }
                    if s.is_null() {
                        while len.wrapping_sub(off) < 3 as size_t {
                            buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                                as *mut ::core::ffi::c_char;
                            len = len.wrapping_mul(2 as size_t);
                        }
                        let fresh14 = off;
                        off = off.wrapping_add(1);
                        *buf.offset(fresh14 as isize) = '#' as i32 as ::core::ffi::c_char;
                        let fresh15 = off;
                        off = off.wrapping_add(1);
                        *buf.offset(fresh15 as isize) = ch as ::core::ffi::c_char;
                        continue;
                    } else {
                        n = strlen(s);
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"found #%c: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            ch,
                            s,
                        );
                        if format_replace(es, s, n, &raw mut buf, &raw mut len, &raw mut off)
                            != 0 as ::core::ffi::c_int
                        {
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"found #%c\0" as *const u8 as *const ::core::ffi::c_char,
                ch,
            );
            while len.wrapping_sub(off) < 2 as size_t {
                buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                    as *mut ::core::ffi::c_char;
                len = len.wrapping_mul(2 as size_t);
            }
            let fresh13 = off;
            off = off.wrapping_add(1);
            *buf.offset(fresh13 as isize) = ch as ::core::ffi::c_char;
        }
    }
    *buf.offset(off as isize) = '\0' as i32 as ::core::ffi::c_char;
    format_log1(
        es,
        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
        b"result is: %s\0" as *const u8 as *const ::core::ffi::c_char,
        buf,
    );
    (*es).loop_0 = (*es).loop_0.wrapping_sub(1);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn format_expand_time(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut es: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    memset(
        &raw mut es as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<format_expand_state>() as size_t,
    );
    es.ft = ft;
    es.flags = FORMAT_EXPAND_TIME;
    es.start_time = get_timer();
    return format_expand1(&raw mut es, fmt);
}
#[no_mangle]
pub unsafe extern "C" fn format_expand(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut es: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    memset(
        &raw mut es as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<format_expand_state>() as size_t,
    );
    es.ft = ft;
    es.flags = 0 as ::core::ffi::c_int;
    es.start_time = get_timer();
    return format_expand1(&raw mut es, fmt);
}
#[no_mangle]
pub unsafe extern "C" fn format_single(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ft = format_create_defaults(item, c, s, wl, wp);
    expanded = format_expand(ft, fmt);
    format_free(ft);
    return expanded;
}
#[no_mangle]
pub unsafe extern "C" fn format_single_from_state(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) -> *mut ::core::ffi::c_char {
    return format_single(item, fmt, c, (*fs).s, (*fs).wl, (*fs).wp);
}
#[no_mangle]
pub unsafe extern "C" fn format_single_from_target(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut tc: *mut client = cmdq_get_target_client(item);
    return format_single_from_state(item, fmt, tc, cmdq_get_target(item));
}

#[cfg(test)]
mod format_choose_tests {
    use super::*;

    #[test]
    fn split_operands_preserve_escapes_nesting_and_bytes() {
        unsafe {
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            let mut es: format_expand_state = std::mem::zeroed();
            es.ft = ft;
            es.start_time = get_timer();

            for (input, expected_left, expected_right) in [
                (
                    b"one#,two,three\0".as_slice(),
                    b"one#,two".as_slice(),
                    b"three".as_slice(),
                ),
                (b"\xff,\xfe\0", b"\xff", b"\xfe"),
                (b"#{?1,a,b},tail\0", b"#{?1,a,b}", b"tail"),
                (b",\0", b"", b""),
            ] {
                let input = CStr::from_bytes_with_nul(input).unwrap();
                let (left, right) = format_choose_loop(&raw mut es, input.as_ptr());
                assert_eq!(left.to_bytes(), expected_left, "{input:?}");
                assert_eq!(right.unwrap().to_bytes(), expected_right, "{input:?}");
            }
            let (all, active) = format_choose_loop(&raw mut es, b"no delimiter\0".as_ptr().cast());
            assert_eq!(all.to_bytes(), b"no delimiter");
            assert!(active.is_none());

            for (input, expected_left, expected_right) in [
                (
                    b"one#,two,three\0".as_slice(),
                    b"one,two".as_slice(),
                    b"three".as_slice(),
                ),
                (b"\xff,\xfe\0", b"\xff", b"\xfe"),
                (b",\0", b"", b""),
            ] {
                let input = CStr::from_bytes_with_nul(input).unwrap();
                let mut left = std::ptr::null_mut();
                let mut right = std::ptr::null_mut();
                assert_eq!(
                    format_choose(&raw mut es, input.as_ptr(), &raw mut left, &raw mut right,),
                    0,
                    "{input:?}"
                );
                assert_eq!(CStr::from_ptr(left).to_bytes(), expected_left, "{input:?}");
                assert_eq!(
                    CStr::from_ptr(right).to_bytes(),
                    expected_right,
                    "{input:?}"
                );
                free(left.cast());
                free(right.cast());
            }

            let mut left = std::ptr::null_mut();
            let mut right = std::ptr::null_mut();
            assert_eq!(
                format_choose(
                    &raw mut es,
                    b"no delimiter\0".as_ptr().cast(),
                    &raw mut left,
                    &raw mut right,
                ),
                -1
            );
            assert!(left.is_null() && right.is_null());
            format_free(ft);
        }
    }
}

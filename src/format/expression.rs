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
pub(super) fn format_quote_shell(s: &CStr) -> CString {
    let input = s.to_bytes();
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
pub(super) fn format_quote_shell_single(s: &CStr) -> CString {
    let input = s.to_bytes();
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
pub(super) fn format_quote_style(s: &CStr) -> CString {
    let input = s.to_bytes();
    let mut quoted = Vec::with_capacity(input.len().saturating_mul(2));
    for &byte in input {
        if byte == b'#' {
            quoted.push(b'#');
        }
        quoted.push(byte);
    }
    CString::new(quoted).expect("style-quoted C string contains no NUL")
}
pub(crate) unsafe fn format_pretty_time_cstring(
    mut t: time_t,
    mut seconds: ::core::ffi::c_int,
) -> CString {
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
        return CStr::from_ptr(s.as_ptr()).to_owned();
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
        return CStr::from_ptr(s.as_ptr()).to_owned();
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
        return CStr::from_ptr(s.as_ptr()).to_owned();
    }
    strftime(
        &raw mut s as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
        b"%h%y\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut tm,
    );
    return CStr::from_ptr(s.as_ptr()).to_owned();
}
pub(super) unsafe fn format_relative_time(mut t: time_t) -> Option<CString> {
    let mut now: time_t = 0;
    let mut age: time_t = 0;
    let mut d: u_int = 0;
    let mut h: u_int = 0;
    let mut m: u_int = 0;
    let mut s: u_int = 0;
    let mut out: [::core::ffi::c_char; 32] = [0; 32];
    time(&raw mut now);
    if t > now {
        return None;
    }
    if t == now {
        return Some(c"0s".to_owned());
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
    return Some(CStr::from_ptr(out.as_ptr()).to_owned());
}
unsafe fn format_time_difference(t: time_t) -> CString {
    CString::new((time(std::ptr::null_mut()) - t).to_string())
        .expect("time difference contains no NUL")
}

pub(super) unsafe fn format_find(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut modifiers: uint64_t,
    mut time_format: *const ::core::ffi::c_char,
) -> Option<CString> {
    let mut current_block: u64;
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_find: format_entry = format_entry {
        owned_cb: None,
        key: Default::default(),
        value: Default::default(),
        time: 0,
    };
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut found: Option<CString> = None;
    let mut s: [::core::ffi::c_char; 512] = [0; 512];
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
    let parsed_option = options_parse_owned(CStr::from_ptr(key));
    if let Some(parsed) = &parsed_option {
        let name = parsed.name.as_ptr();
        o = options_get(global_options, name);
        if o.is_null() && !(*ft).wp.is_null() {
            o = options_get((*(*ft).wp).options, name);
        }
        if o.is_null() && !(*ft).w.is_null() {
            o = options_get((*(*ft).w).options, name);
        }
        if o.is_null() {
            o = options_get(global_w_options, name);
        }
        if o.is_null() && !(*ft).s.is_null() {
            o = options_get((*(*ft).s).options, name);
        }
        if o.is_null() {
            o = options_get(global_s_options, name);
        }
    }
    if !o.is_null() {
        let array_key = parsed_option
            .as_ref()
            .and_then(|parsed| parsed.array_key.as_ref())
            .map_or(::core::ptr::null(), |key| key.as_ptr());
        found = Some(options_to_cstring(o, array_key, 1 as ::core::ffi::c_int));
    } else {
        if let Some(entry) = format_table_get(CStr::from_ptr(key)) {
            match entry.get(ft) {
                Some(FormatValue::String(value)) => found = Some(value),
                Some(FormatValue::Time(value)) => t = value,
                None => {}
            }
        } else {
            fe_find.key = ::std::ffi::CStr::from_ptr(key as *mut ::core::ffi::c_char).to_owned();
            fe = format_entry_tree_find(&raw mut (*ft).tree, &raw mut fe_find);
            if !fe.is_null() {
                if (*fe).time != 0 as time_t {
                    t = (*fe).time;
                } else {
                    format_entry_ensure_value(ft, fe);
                    found = Some(
                        CStr::from_ptr(
                            ((*fe).value)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_owned(),
                    );
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
                    if !envent.is_null() && !(*envent).value.is_none() {
                        found = Some(
                            CStr::from_ptr(
                                ((*envent).value)
                                    .as_ref()
                                    .map_or(::core::ptr::null_mut(), |value| {
                                        value.as_ptr().cast_mut()
                                    }),
                            )
                            .to_owned(),
                        );
                        current_block = 11739001764845178280;
                    } else {
                        current_block = 1836292691772056875;
                    }
                } else {
                    current_block = 1836292691772056875;
                }
                match current_block {
                    11739001764845178280 => {}
                    _ => return None,
                }
            }
        }
    }
    if modifiers & FORMAT_TIMESTRING as uint64_t != 0 {
        if t == 0 as time_t && found.is_some() {
            t = strtonum(
                found.as_ref().unwrap().as_ptr(),
                0 as ::core::ffi::c_longlong,
                INT64_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as time_t;
            if !errstr.is_null() {
                t = 0 as time_t;
            }
            found = None;
        }
        if t == 0 as time_t {
            return None;
        }
        if modifiers & FORMAT_RELATIVE as uint64_t != 0 {
            found = format_relative_time(t);
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_DIFFERENCE != 0 {
            found = Some(format_time_difference(t));
        } else if modifiers & FORMAT_PRETTY as uint64_t != 0 {
            found = Some(format_pretty_time_cstring(t, 0));
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
            found = Some(CStr::from_ptr(s.as_ptr()).to_owned());
        }
        return found;
    }
    let mut found = if t != 0 {
        CString::new(t.to_string()).expect("timestamp contains no NUL")
    } else {
        found?
    };
    if modifiers & FORMAT_BASENAME as uint64_t != 0 {
        // basename/dirname may mutate their input or return a static string.
        let mut scratch = found.into_bytes_with_nul();
        found = CStr::from_ptr(__xpg_basename(scratch.as_mut_ptr().cast())).to_owned();
    }
    if modifiers & FORMAT_DIRNAME as uint64_t != 0 {
        let mut scratch = found.into_bytes_with_nul();
        found = CStr::from_ptr(dirname(scratch.as_mut_ptr().cast())).to_owned();
    }
    if modifiers & FORMAT_QUOTE_SHELL as uint64_t != 0 {
        found = format_quote_shell(found.as_c_str());
    }
    if modifiers & FORMAT_QUOTE_SHELL_SQ as uint64_t != 0 {
        found = format_quote_shell_single(found.as_c_str());
    }
    if modifiers & FORMAT_QUOTE_STYLE as uint64_t != 0 {
        found = format_quote_style(found.as_c_str());
    }
    if modifiers & FORMAT_QUOTE_ARGUMENTS as uint64_t != 0 {
        found = args_escape_cstring(found.as_c_str());
    }
    Some(found)
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

unsafe fn format_strip_cstring(es: *mut format_expand_state, s: &CStr) -> CString {
    let input = s.to_bytes();
    let mut out = Vec::with_capacity(input.len());
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    for (index, &byte) in input.iter().enumerate() {
        if format_check_time(es, &raw mut check) == 0 {
            return CString::default();
        }
        let next = input.get(index + 1).copied().unwrap_or(0);
        if byte == b'#' && next == b'{' {
            brackets += 1;
        }
        if byte == b'#' && (next == 0 || b",#{}:".contains(&next)) {
            if brackets != 0 as ::core::ffi::c_int {
                out.push(byte);
            }
        } else {
            if byte == b'}' {
                brackets -= 1;
            }
            out.push(byte);
        }
    }
    CString::new(out).expect("stripped C string contains no NUL")
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
unsafe fn format_choose(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
) -> Option<(CString, CString)> {
    let cp = format_skip1(es, s, b",\0" as *const u8 as *const ::core::ffi::c_char);
    if cp.is_null() {
        return None;
    }
    let split = cp.offset_from(s) as usize;
    // Clone both operands before expansion: a format callback may reenter
    // the formatter or change the storage backing the original input.
    let left0 = CString::new(&CStr::from_ptr(s).to_bytes()[..split]).unwrap();
    let right0 = CStr::from_ptr(cp.add(1)).to_owned();
    let left = format_expand1_cstring(es, left0.as_ptr());
    drop(left0);
    let right = format_expand1_cstring(es, right0.as_ptr());
    Some((left, right))
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
unsafe fn format_expand_modifier_arg(
    es: *mut format_expand_state,
    value: *const ::core::ffi::c_char,
) -> CString {
    format_expand1_cstring(es, value)
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
pub(super) unsafe fn format_match_fuzzy(
    pattern: &CStr,
    text: &CStr,
    mut positions: ::core::ffi::c_int,
) -> CString {
    let mut width: u_int = 0;
    width = format_width(text.as_ptr());
    if width == 0 as u_int {
        width = 1 as u_int;
    }
    let Some(bs) = fuzzy_match_owned(pattern, text, width, None) else {
        return if positions != 0 {
            CString::default()
        } else {
            c"0".to_owned()
        };
    };
    if positions == 0 {
        return c"1".to_owned();
    }
    let mut indexes = Vec::new();
    for i in 0..width {
        if bs[(i >> 3) as usize] & (1 << (i & 7)) != 0 {
            indexes.push(i.to_string());
        }
    }
    CString::new(indexes.join(",")).expect("fuzzy positions contain no NUL")
}

pub(super) unsafe fn format_match(
    mut fm: *mut format_modifier,
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
) -> CString {
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
        return format_match_fuzzy(
            CStr::from_ptr(pattern),
            CStr::from_ptr(text),
            1 as ::core::ffi::c_int,
        );
    }
    if !strchr(s, 'z' as i32).is_null() {
        return format_match_fuzzy(
            CStr::from_ptr(pattern),
            CStr::from_ptr(text),
            0 as ::core::ffi::c_int,
        );
    }
    if strchr(s, 'r' as i32).is_null() {
        if !strchr(s, 'i' as i32).is_null() {
            flags |= FNM_CASEFOLD;
        }
        if fnmatch(pattern, text, flags) != 0 as ::core::ffi::c_int {
            return c"0".to_owned();
        }
    } else {
        flags = REG_EXTENDED | REG_NOSUB;
        if !strchr(s, 'i' as i32).is_null() {
            flags |= REG_ICASE;
        }
        if regcomp(&raw mut r, pattern, flags) != 0 as ::core::ffi::c_int {
            return c"0".to_owned();
        }
        let regex_owner = crate::src::regsub::CompiledRegex::new(&raw mut r);
        if regexec(
            &raw mut r,
            text,
            0 as size_t,
            ::core::ptr::null_mut::<regmatch_t>(),
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            drop(regex_owner);
            return c"0".to_owned();
        }
        drop(regex_owner);
    }
    return c"1".to_owned();
}
pub(super) unsafe fn format_sub(
    mut fm: *mut format_modifier,
    mut text: *const ::core::ffi::c_char,
    mut pattern: *const ::core::ffi::c_char,
    mut with: *const ::core::ffi::c_char,
) -> CString {
    let mut flags: ::core::ffi::c_int = REG_EXTENDED;
    if (*fm).argc() >= 3 as ::core::ffi::c_int && !strchr((*fm).arg(2), 'i' as i32).is_null() {
        flags |= REG_ICASE;
    }
    regsub_cstring(
        CStr::from_ptr(pattern),
        CStr::from_ptr(with),
        CStr::from_ptr(text),
        flags,
    )
    .unwrap_or_else(|| CStr::from_ptr(text).to_owned())
}

pub(super) unsafe fn format_search(
    mut fm: *mut format_modifier,
    mut wp: *mut window_pane,
    mut s: *const ::core::ffi::c_char,
) -> CString {
    let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regex: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fm).argc() >= 1 as ::core::ffi::c_int {
        if !strchr((*fm).arg(0), 'i' as i32).is_null() {
            ignore = 1 as ::core::ffi::c_int;
        }
        if !strchr((*fm).arg(0), 'r' as i32).is_null() {
            regex = 1 as ::core::ffi::c_int;
        }
    }
    CString::new(window_pane_search(wp, s, regex, ignore).to_string())
        .expect("search count contains no NUL")
}

pub(super) unsafe fn format_bool_op_1(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut not: ::core::ffi::c_int,
) -> CString {
    let mut result: ::core::ffi::c_int = 0;
    let expanded = format_expand1_cstring(es, fmt);
    result = format_true(expanded.as_ptr());
    if not != 0 {
        result = (result == 0) as ::core::ffi::c_int;
    }
    return if result != 0 {
        c"1".to_owned()
    } else {
        c"0".to_owned()
    };
}
pub(super) unsafe fn format_bool_op_n(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut and: ::core::ffi::c_int,
) -> CString {
    let mut result: ::core::ffi::c_int = 0;
    let mut cp1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
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
        let expanded = format_expand1_cstring(es, operand.as_ptr());
        format_log1(
            es,
            b"format_bool_op_n\0" as *const u8 as *const ::core::ffi::c_char,
            b"operator %s has operand: %s\0" as *const u8 as *const ::core::ffi::c_char,
            if and != 0 {
                b"&&\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"||\0" as *const u8 as *const ::core::ffi::c_char
            },
            expanded.as_ptr(),
        );
        if and != 0 {
            result = (result != 0 && format_true(expanded.as_ptr()) != 0) as ::core::ffi::c_int;
        } else {
            result = (result != 0 || format_true(expanded.as_ptr()) != 0) as ::core::ffi::c_int;
        }
        if cp2.is_null() {
            break;
        }
        cp1 = cp2.offset(1 as ::core::ffi::c_int as isize);
    }
    return if result != 0 {
        c"1".to_owned()
    } else {
        c"0".to_owned()
    };
}
pub(super) unsafe fn format_session_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let name = format_expand1_cstring(es, fmt);
    s = sessions_minmax(&*std::ptr::addr_of!(sessions), RB_NEGINF);
    while !s.is_null() {
        if strcmp(((*s).name).as_ptr().cast_mut(), name.as_ptr()) == 0 as ::core::ffi::c_int {
            return c"1".to_owned();
        }
        s = sessions_next(&*s);
    }
    return c"0".to_owned();
}
pub(super) unsafe fn format_loop_sessions(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
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
    let mut buffer = Vec::new();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: ::core::ffi::c_int = 0;
    let (all, active) = format_choose_loop(es, fmt);
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
        let expanded = format_expand1_cstring(&raw mut next, use_0);
        format_free(next.ft);
        buffer.extend_from_slice(expanded.as_bytes());
        i += 1;
    }
    drop(active);
    drop(all);
    CString::new(buffer).expect("format loop output contains no NUL")
}

pub(super) unsafe fn format_window_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> Option<CString> {
    let mut ft: *mut format_tree = (*es).ft;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*ft).s.is_null() {
        format_log1(
            es,
            b"format_window_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"window name but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return None;
    }
    let name = format_expand1_cstring(es, fmt);
    wl = winlinks_minmax(&(*(*ft).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp((*(*wl).window).name.as_ptr(), name.as_ptr()) == 0 as ::core::ffi::c_int {
            return Some(c"1".to_owned());
        }
        wl = winlinks_next(&*wl);
    }
    return Some(c"0".to_owned());
}
pub(super) unsafe extern "C" fn format_add_window_neighbour(
    mut nft: *mut format_tree,
    mut wl: *mut winlink,
    mut s: *mut session,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
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
            let oval = options_to_cstring(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                1 as ::core::ffi::c_int,
            );
            format_add(
                nft,
                prefixed.as_ptr(),
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                oval.as_ptr(),
            );
        }
        o = options_next(o);
    }
}
pub(super) unsafe fn format_loop_windows(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> Option<CString> {
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
    let mut buffer = Vec::new();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut i: ::core::ffi::c_int = 0;
    if s.is_null() {
        format_log1(
            es,
            b"format_loop_windows\0" as *const u8 as *const ::core::ffi::c_char,
            b"window loop but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return None;
    }
    let (all, active) = format_choose_loop(es, fmt);
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
        let expanded = format_expand1_cstring(&raw mut next, use_0);
        format_free(nft);
        buffer.extend_from_slice(expanded.as_bytes());
        i += 1;
    }
    drop(active);
    drop(all);
    Some(CString::new(buffer).expect("format loop output contains no NUL"))
}

pub(super) unsafe fn format_loop_panes(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> Option<CString> {
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
    let mut buffer = Vec::new();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    if (*ft).w.is_null() {
        format_log1(
            es,
            b"format_loop_panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"pane loop but no window\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return None;
    }
    let (all, active) = format_choose_loop(es, fmt);
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
        let expanded = format_expand1_cstring(&raw mut next, use_0);
        format_free(nft);
        buffer.extend_from_slice(expanded.as_bytes());
        i += 1;
    }
    drop(active);
    drop(all);
    Some(CString::new(buffer).expect("format loop output contains no NUL"))
}

pub(super) unsafe fn format_loop_add_option(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    buffer: &mut Vec<u8>,
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
    let s = options_to_cstring(
        o,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    drop(s);
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
    let expanded = format_expand1_cstring(&raw mut next, fmt);
    format_free(nft);
    buffer.extend_from_slice(expanded.as_bytes());
}
pub(super) unsafe fn format_loop_add_array_item(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    buffer: &mut Vec<u8>,
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
    let s = options_to_cstring(o, array_key, 0 as ::core::ffi::c_int);
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    drop(s);
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
    let expanded = format_expand1_cstring(&raw mut next, fmt);
    format_free(nft);
    buffer.extend_from_slice(expanded.as_bytes());
}
pub(super) unsafe fn format_loop_options(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> CString {
    let mut ft: *mut format_tree = (*es).ft;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut buffer = Vec::new();
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
        return c"".to_owned();
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
            format_loop_add_option(es, fmt, &mut buffer, o, n, i);
            i = i.wrapping_add(1);
            o = options_next(o);
        } else {
            a = options_array_first(o);
            while !a.is_null() {
                format_loop_add_array_item(es, fmt, &mut buffer, o, a, n as ::core::ffi::c_int, i);
                i = i.wrapping_add(1);
                a = options_array_next(a);
            }
            o = options_next(o);
        }
    }
    CString::new(buffer).expect("format loop output contains no NUL")
}

pub(super) unsafe fn format_loop_environ(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> CString {
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
    let mut buffer = Vec::new();
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
        return c"".to_owned();
    }
    envent = environ_first(env);
    while !envent.is_null() {
        format_log1(
            es,
            b"format_loop_environ\0" as *const u8 as *const ::core::ffi::c_char,
            b"environment loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).name).as_ptr().cast_mut(),
        );
        nft = format_create(c, item, FORMAT_NONE, (*ft).flags);
        format_add(
            nft,
            b"environ_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).name).as_ptr().cast_mut(),
        );
        if (*envent).value.is_none() {
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
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
            ((*envent).value
                == if (NULL_0 as *mut ::core::ffi::c_char).is_null() {
                    None
                } else {
                    Some(::std::ffi::CStr::from_ptr(NULL_0 as *mut ::core::ffi::c_char).to_owned())
                }) as ::core::ffi::c_int,
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
        let expanded = format_expand1_cstring(&raw mut next, fmt);
        format_free(nft);
        buffer.extend_from_slice(expanded.as_bytes());
        i = i.wrapping_add(1);
        envent = environ_next(envent);
    }
    CString::new(buffer).expect("format loop output contains no NUL")
}

pub(super) unsafe fn format_loop_clients(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
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
    let mut buffer = Vec::new();
    let mut i: ::core::ffi::c_int = 0;
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
            ((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
        let expanded = format_expand1_cstring(&raw mut next, fmt);
        format_free(nft);
        buffer.extend_from_slice(expanded.as_bytes());
        i += 1;
    }
    CString::new(buffer).expect("format loop output contains no NUL")
}

unsafe fn format_float(value: f64, precision: ::core::ffi::c_int) -> CString {
    let length = libc::snprintf(std::ptr::null_mut(), 0, c"%.*f".as_ptr(), precision, value);
    if length < 0 {
        fatalx(c"format number failed".as_ptr());
    }
    let mut output = vec![0u8; length as usize + 1];
    libc::snprintf(
        output.as_mut_ptr().cast(),
        output.len(),
        c"%.*f".as_ptr(),
        precision,
        value,
    );
    CString::from_vec_with_nul(output).expect("formatted number contains one terminating NUL")
}
pub(super) unsafe fn format_replace_expression(
    mut mexp: *mut format_modifier,
    mut es: *mut format_expand_state,
    mut copy: *const ::core::ffi::c_char,
) -> Option<CString> {
    let mut current_block: u64;
    let mut argc: ::core::ffi::c_int = (*mexp).argc();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut endch: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
                    let operands = format_choose(es, copy);
                    if operands.is_none() {
                        format_log1(
                            es,
                            b"format_replace_expression\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"expression syntax error\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        let (left, right) = operands.unwrap();
                        mleft = strtod(left.as_ptr(), &raw mut endch);
                        if *endch as ::core::ffi::c_int != '\0' as i32 {
                            format_log1(
                                es,
                                b"format_replace_expression\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"expression left side is invalid: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                left.as_ptr(),
                            );
                        } else {
                            mright = strtod(right.as_ptr(), &raw mut endch);
                            if *endch as ::core::ffi::c_int != '\0' as i32 {
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression right side is invalid: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    right.as_ptr(),
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
                                let value = format_float(
                                    if use_fp != 0 {
                                        result
                                    } else {
                                        result as ::core::ffi::c_longlong as f64
                                    },
                                    prec as ::core::ffi::c_int,
                                );
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression result is %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    value.as_ptr(),
                                );
                                return Some(value);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    return None;
}
pub(super) unsafe extern "C" fn format_cycle_callback(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    if (*c).message_string.is_none() && (*c).prompt.is_null() {
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
    if event_initialized(&(*c).cycle_timer) == 0 {
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
pub(super) unsafe fn format_cycle(
    mut es: *mut format_expand_state,
    mut frames: *const ::core::ffi::c_char,
    mut count: u_int,
) -> CString {
    let mut ft: *mut format_tree = (*es).ft;
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut index: u_int = 0;
    let mut i: u_int = 0;
    if (*ft).flags & FORMAT_STATUS == 0 || (*es).flags & FORMAT_EXPAND_NOCYCLE != 0 {
        return c"".to_owned();
    }
    if *frames as ::core::ffi::c_int == '\0' as i32 {
        return c"".to_owned();
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
    return CString::new(std::slice::from_raw_parts(
        start.cast::<u8>(),
        end.offset_from(start) as usize,
    ))
    .expect("cycle frame contains no NUL");
}
pub(super) unsafe fn format_replace(
    mut es: *mut format_expand_state,
    mut key: *const ::core::ffi::c_char,
    mut keylen: size_t,
    output: &mut Vec<u8>,
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
    let mut time_format: Option<CString> = None;
    let mut value = CString::default();
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
                            time_format =
                                Some(format_strip_cstring(es, CStr::from_ptr((*fm).arg(1))));
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
            value = c"".to_owned();
        } else {
            if modifiers & FORMAT_CLIENT_TERMCAP as uint64_t != 0 {
                if tty_term_has_name((*(*ft).c).tty.term, copy) != 0 {
                    value = c"1".to_owned();
                } else {
                    value = c"0".to_owned();
                }
            }
            if modifiers & FORMAT_CLIENT_TERMFEAT as uint64_t != 0 {
                if tty_feature_present((*(*ft).c).tty.term, copy) != 0 {
                    value = c"1".to_owned();
                } else {
                    value = c"0".to_owned();
                }
            }
            if modifiers & FORMAT_CLIENT_ENVIRON as uint64_t != 0 {
                envent = environ_find((*(*ft).c).environ, copy);
                if !envent.is_null() && !(*envent).value.is_none() {
                    value = CStr::from_ptr(
                        ((*envent).value)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )
                    .to_owned();
                } else {
                    value = c"".to_owned();
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
            value.as_ptr(),
        );
    } else if modifiers & FORMAT_LITERAL as uint64_t != 0 {
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"literal string is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
        );
        value = format_unescape_cstring(es, copy, strlen(copy));
    } else if modifiers & FORMAT_CHARACTER as uint64_t != 0 {
        let new = format_expand1_cstring(es, copy);
        c = strtonum(
            new.as_ptr(),
            32 as ::core::ffi::c_longlong,
            126 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            value = c"".to_owned();
        } else {
            value = CString::new(vec![c as u8]).expect("printable character contains no NUL");
        }
    } else if modifiers & FORMAT_COLOUR as uint64_t != 0 {
        let new = format_expand1_cstring(es, copy);
        if modifiers & (FORMAT_COLOUR_ESC_FG | FORMAT_COLOUR_ESC_BG) as uint64_t != 0 {
            if strcasecmp(
                new.as_ptr(),
                b"none\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                value = c"\x1B[0m".to_owned();
            } else {
                c = colour_parse_cstr(new.as_c_str()).unwrap_or(-1);
                if c == -(1 as ::core::ffi::c_int) {
                    value = c"".to_owned();
                } else {
                    let escape = colour_format_escape_for_client(
                        (*ft).c.as_ref(),
                        c,
                        modifiers & FORMAT_COLOUR_ESC_BG as uint64_t != 0,
                    );
                    value = escape.unwrap_or_default();
                }
            }
        } else {
            c = colour_parse_cstr(new.as_c_str()).unwrap_or(-1);
            if c == -(1 as ::core::ffi::c_int) || {
                c = colour_force_rgb(c);
                c == -(1 as ::core::ffi::c_int)
            } {
                value = c"".to_owned();
            } else {
                value = CString::new(format!("{:06x}", c & 0xffffff)).expect("RGB contains no NUL");
            }
        }
    } else {
        if modifiers & FORMAT_SESSIONS as uint64_t != 0 {
            value = format_loop_sessions(es, copy);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_WINDOWS as uint64_t != 0 {
            if let Some(result) = format_loop_windows(es, copy) {
                value = result;
                current_block = 1803726662341650892;
            } else {
                current_block = 6506207624831006569;
            }
        } else if modifiers & FORMAT_PANES as uint64_t != 0 {
            if let Some(result) = format_loop_panes(es, copy) {
                value = result;
                current_block = 1803726662341650892;
            } else {
                current_block = 6506207624831006569;
            }
        } else if modifiers & FORMAT_CLIENTS as uint64_t != 0 {
            value = format_loop_clients(es, copy);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_OPTIONS as uint64_t != 0 {
            value = format_loop_options(es, copy, loop_flags);
            current_block = 1803726662341650892;
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_ENVIRON != 0 {
            value = format_loop_environ(es, copy, loop_flags);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_WINDOW_NAME as uint64_t != 0 {
            if let Some(result) = format_window_name(es, copy) {
                value = result;
                current_block = 1803726662341650892;
            } else {
                current_block = 6506207624831006569;
            }
        } else if modifiers & FORMAT_SESSION_NAME as uint64_t != 0 {
            value = format_session_name(es, copy);
            current_block = 1803726662341650892;
        } else if !search.is_null() {
            let new = format_expand1_cstring(es, copy);
            if wp.is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' but no pane\0" as *const u8 as *const ::core::ffi::c_char,
                    new.as_ptr(),
                );
                value = c"0".to_owned();
            } else {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    new.as_ptr(),
                    (*wp).id,
                );
                value = format_search(search, wp, new.as_ptr());
            }
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_REPEAT as uint64_t != 0 {
            let operands = format_choose(es, copy);
            if operands.is_none() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"repeat syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                let (left, right) = operands.unwrap();
                nrep = strtonum(
                    right.as_ptr(),
                    1 as ::core::ffi::c_longlong,
                    FORMAT_MAX_REPEAT as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    value = CString::default();
                    current_block = 1803726662341650892;
                } else {
                    let mut repeated = Vec::new();
                    current_block = 1803726662341650892;
                    for _ in 0..nrep {
                        if format_check_time(es, &raw mut check) == 0 {
                            current_block = 6506207624831006569;
                            break;
                        }
                        repeated.extend_from_slice(left.as_bytes());
                    }
                    value = CString::new(repeated).expect("repeated C string contains no NUL");
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
            let operands = format_choose(es, copy);
            if operands.is_none() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                let (left, right) = operands.unwrap();
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s left is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    left.as_ptr(),
                );
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s right is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    right.as_ptr(),
                );
                if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"==\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) == 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"!=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) != 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) < 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) > 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) <= 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left.as_ptr(), right.as_ptr()) >= 0 as ::core::ffi::c_int {
                        value = c"1".to_owned();
                    } else {
                        value = c"0".to_owned();
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"m\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    value = format_match(cmp, left.as_ptr(), right.as_ptr());
                }
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
                        value = format_expand1_cstring(es, cp);
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
                        let found = format_find(
                            ft,
                            condition.as_ptr(),
                            modifiers,
                            time_format
                                .as_ref()
                                .map_or(::core::ptr::null(), |s| s.as_ptr()),
                        );
                        let condition_is_true = if found.is_none() {
                            let expanded = format_expand1_cstring(es, condition.as_ptr());
                            if expanded.as_c_str() == condition.as_c_str() {
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"condition '%s' not found; assuming false\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    condition.as_ptr(),
                                );
                                false
                            } else {
                                format_true(expanded.as_ptr()) != 0
                            }
                        } else {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' found: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition.as_ptr(),
                                found.as_ref().unwrap().as_ptr(),
                            );
                            format_true(found.as_ref().unwrap().as_ptr()) != 0
                        };
                        drop(found);
                        cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                        cp2 =
                            format_skip1(es, cp, b",\0" as *const u8 as *const ::core::ffi::c_char);
                        if condition_is_true {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' is true\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition.as_ptr(),
                            );
                            if cp2.is_null() {
                                value = format_expand1_cstring(es, cp);
                            } else {
                                let right = CString::new(std::slice::from_raw_parts(
                                    cp.cast::<u8>(),
                                    cp2.offset_from(cp) as usize,
                                ))
                                .expect("format branch contains no NUL");
                                value = format_expand1_cstring(es, right.as_ptr());
                                drop(right);
                            }
                            drop(condition);
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
                            if cp2.is_null() {
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"no condition matched in '%s'; using empty string\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                    copy.offset(1 as ::core::ffi::c_int as isize),
                                );
                                value = c"".to_owned();
                                break;
                            } else {
                                cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                            }
                        }
                    }
                }
            } else if !mexp.is_null() {
                value = format_replace_expression(mexp, es, copy).unwrap_or_default();
            } else if !strstr(copy, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"expanding inner format '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                value = format_expand1_cstring(es, copy);
            } else {
                value = if let Some(found) = format_find(
                    ft,
                    copy,
                    modifiers,
                    time_format
                        .as_ref()
                        .map_or(std::ptr::null(), |s| s.as_ptr()),
                ) {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"format '%s' found: %s".as_ptr(),
                        copy,
                        found.as_ptr(),
                    );
                    found
                } else {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"format '%s' not found".as_ptr(),
                        copy,
                    );
                    CString::default()
                };
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
                return -(1 as ::core::ffi::c_int);
            }
        }
    }
    if modifiers & FORMAT_EXPAND as uint64_t != 0 {
        value = format_expand1_cstring(es, value.as_ptr());
    } else if modifiers & FORMAT_EXPANDTIME as uint64_t != 0 {
        format_copy_state(&raw mut next, es, FORMAT_EXPAND_TIME);
        value = format_expand1_cstring(&raw mut next, value.as_ptr());
    }
    for &modifier in &sub {
        let left = format_expand1_cstring(es, (*modifier).arg(0));
        let right = format_expand1_cstring(es, (*modifier).arg(1));
        value = format_sub(modifier, value.as_ptr(), left.as_ptr(), right.as_ptr());
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"substitute '%s' to '%s': %s".as_ptr(),
            left.as_ptr(),
            right.as_ptr(),
            value.as_ptr(),
        );
    }
    if limit != 0 {
        let mut trimmed = if limit > 0 {
            format_trim_left_bytes(value.as_c_str(), limit as u_int)
        } else {
            format_trim_right_bytes(value.as_c_str(), -limit as u_int)
        };
        if !marker.is_null() && trimmed != value.as_bytes() {
            let marker = CStr::from_ptr(marker).to_bytes();
            if limit > 0 {
                trimmed.extend_from_slice(marker);
            } else {
                let mut prefixed = Vec::with_capacity(marker.len() + trimmed.len());
                prefixed.extend_from_slice(marker);
                prefixed.extend_from_slice(&trimmed);
                trimmed = prefixed;
            }
        }
        value = CString::new(trimmed).expect("trimmed C string contains no NUL");
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"applied length limit %d: %s".as_ptr(),
            limit,
            value.as_ptr(),
        );
    }
    if width != 0 {
        value = utf8_pad_cstring(value.as_c_str(), width.unsigned_abs(), width < 0);
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"applied padding width %d: %s".as_ptr(),
            width,
            value.as_ptr(),
        );
    }
    if modifiers & FORMAT_LENGTH as uint64_t != 0 {
        value = CString::new(value.as_bytes().len().to_string()).expect("length contains no NUL");
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"replacing with length: %s".as_ptr(),
            value.as_ptr(),
        );
    }
    if modifiers & FORMAT_WIDTH as uint64_t != 0 {
        value =
            CString::new(format_width(value.as_ptr()).to_string()).expect("width contains no NUL");
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"replacing with width: %s".as_ptr(),
            value.as_ptr(),
        );
    }
    output.extend_from_slice(value.as_bytes());
    format_log1(
        es,
        c"format_replace".as_ptr(),
        c"replaced '%s' with '%s'".as_ptr(),
        copy0.as_ptr(),
        value.as_ptr(),
    );
    0
}
pub(super) unsafe fn format_expand1_cstring(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
    let mut ft: *mut format_tree = (*es).ft;
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style_end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 0;
    let mut expanded: [::core::ffi::c_char; 8192] = [0; 8192];
    if fmt.is_null()
        || *fmt as ::core::ffi::c_int == '\0' as i32
        || format_check_time(es, ::core::ptr::null_mut::<u_int>()) == 0
    {
        return CString::default();
    }
    if (*es).loop_0 == FORMAT_LOOP_LIMIT as u_int {
        format_log1(
            es,
            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
            b"reached loop limit (%u)\0" as *const u8 as *const ::core::ffi::c_char,
            FORMAT_LOOP_LIMIT,
        );
        return CString::default();
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
            return CString::default();
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
    let mut output = Vec::<u8>::with_capacity(64);
    while *fmt as ::core::ffi::c_int != '\0' as i32 {
        if *fmt as ::core::ffi::c_int != '#' as i32 {
            let fresh10 = fmt;
            fmt = fmt.offset(1);
            output.push(*fresh10 as u8);
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
                    let out = if (*ft).flags & FORMAT_NOJOBS != 0
                        || (*es).flags & FORMAT_EXPAND_NOJOBS != 0
                    {
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() is disabled\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        CString::default()
                    } else {
                        let out = format_job_get(es, name.as_ptr());
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() result: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            out.as_ptr(),
                        );
                        out
                    };
                    output.extend_from_slice(out.as_bytes());
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
                    if format_replace(es, fmt, n, &mut output) != 0 as ::core::ffi::c_int {
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
                        output.extend_from_slice(std::slice::from_raw_parts(
                            fmt.offset(-(2 as ::core::ffi::c_int as isize)).cast::<u8>(),
                            n.wrapping_add(1 as size_t),
                        ));
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
                        output.push(b'#');
                        output.push(ch as u8);
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
                        if format_replace(es, s, n, &mut output) != 0 as ::core::ffi::c_int {
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
            output.push(ch as u8);
        }
    }
    // Literal bytes stop at NUL and replacements append their C-string view.
    let buf = CString::new(output).expect("format expansion contains no NUL");
    format_log1(
        es,
        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
        b"result is: %s\0" as *const u8 as *const ::core::ffi::c_char,
        buf.as_ptr(),
    );
    (*es).loop_0 = (*es).loop_0.wrapping_sub(1);
    return buf;
}
/// Expand a time-aware format into Rust-owned storage.
pub(crate) unsafe fn format_expand_time_cstring(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
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
    format_expand1_cstring(&raw mut es, fmt)
}
/// Own the expanded bytes independently of the tree. A null format is empty.
pub unsafe fn format_expand_cstring(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> CString {
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
    return format_expand1_cstring(&raw mut es, fmt);
}
pub(crate) unsafe fn format_single_cstring(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> CString {
    let ft = format_create_defaults(item, c, s, wl, wp);
    let expanded = format_expand_cstring(ft, fmt);
    format_free(ft);
    return expanded;
}
pub(crate) unsafe fn format_single_from_state_cstring(
    item: *mut cmdq_item,
    fmt: *const ::core::ffi::c_char,
    c: *mut client,
    fs: *mut cmd_find_state,
) -> CString {
    format_single_cstring(item, fmt, c, (*fs).s, (*fs).wl, (*fs).wp)
}
pub(crate) unsafe fn format_single_from_target_cstring(
    item: *mut cmdq_item,
    fmt: *const ::core::ffi::c_char,
) -> CString {
    let tc = cmdq_get_target_client(item);
    format_single_from_state_cstring(item, fmt, tc, cmdq_get_target(item))
}

#[cfg(test)]
mod format_choose_tests {
    use super::*;

    #[test]
    fn expanded_owners_outlive_their_inputs_and_tree() {
        unsafe {
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            let input = CString::new(b"\xff:##:#,:#}:tail#".to_vec()).unwrap();
            let owned = format_expand_cstring(ft, input.as_ptr());
            let timed_owned = format_expand_time_cstring(ft, input.as_ptr());
            drop(input);
            format_free(ft);

            assert_eq!(owned.as_bytes(), b"\xff:#:,:}:tail");
            assert_eq!(timed_owned.as_bytes(), owned.as_bytes());
            // The independent owner remains valid after the tree is freed.
            assert_eq!(owned.as_bytes(), b"\xff:#:,:}:tail");
        }
    }

    #[test]
    fn owned_expansion_preserves_empty_and_limit_results() {
        unsafe {
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            assert!(format_expand_cstring(ft, std::ptr::null())
                .as_bytes()
                .is_empty());
            assert!(format_expand_cstring(ft, c"".as_ptr())
                .as_bytes()
                .is_empty());
            // The first NUL still terminates the input, including invalid bytes after it.
            assert_eq!(
                format_expand_cstring(ft, b"first\0\xffignored\0".as_ptr().cast()).as_bytes(),
                b"first"
            );
            let mut es: format_expand_state = Default::default();
            es.ft = ft;
            es.start_time = get_timer();
            es.loop_0 = FORMAT_LOOP_LIMIT as u_int;
            assert!(format_expand1_cstring(&raw mut es, c"limit".as_ptr())
                .as_bytes()
                .is_empty());
            assert_eq!(es.loop_0, FORMAT_LOOP_LIMIT as u_int);
            es.loop_0 = 0;
            es.start_time = get_timer().wrapping_sub(FORMAT_TIME_LIMIT as uint64_t);
            assert!(format_expand1_cstring(&raw mut es, c"timeout".as_ptr())
                .as_bytes()
                .is_empty());
            es.start_time = get_timer();
            es.flags = FORMAT_EXPAND_TIME;
            let too_long = CString::new(format!("%Y{}", "x".repeat(8192))).unwrap();
            assert!(format_expand1_cstring(&raw mut es, too_long.as_ptr())
                .as_bytes()
                .is_empty());
            format_free(ft);
        }
    }

    #[test]
    fn split_operands_preserve_escapes_nesting_and_bytes() {
        unsafe {
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            let mut es: format_expand_state = Default::default();
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
                let (left, right) = format_choose(&raw mut es, input.as_ptr()).unwrap();
                assert_eq!(left.to_bytes(), expected_left, "{input:?}");
                assert_eq!(right.to_bytes(), expected_right, "{input:?}");
            }

            assert!(format_choose(&raw mut es, c"no delimiter".as_ptr()).is_none());
            format_free(ft);
        }
    }
}

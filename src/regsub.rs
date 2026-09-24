use crate::src::ffi::libc::{regcomp, regexec, regfree};
pub use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
};
use std::ffi::{CStr, CString};

pub(crate) struct CompiledRegex(*mut regex_t);

impl CompiledRegex {
    pub(crate) unsafe fn new(regex: *mut regex_t) -> Self {
        Self(regex)
    }
}

impl Drop for CompiledRegex {
    fn drop(&mut self) {
        unsafe { regfree(self.0) }
    }
}

fn regsub_copy(buf: &mut Vec<u8>, text: &[u8], start: usize, end: usize) {
    buf.extend_from_slice(&text[start..end]);
}
fn regsub_expand(buf: &mut Vec<u8>, with: &[u8], text: &[u8], matches: &[regmatch_t]) {
    let mut index = 0;
    while index < with.len() {
        if with[index] == b'\\' && index + 1 < with.len() {
            let escaped = with[index + 1];
            if escaped.is_ascii_digit() {
                let group = (escaped - b'0') as usize;
                if let Some(matched) = matches.get(group) {
                    if matched.rm_so >= 0 && matched.rm_eo >= 0 && matched.rm_so != matched.rm_eo {
                        let start = usize::try_from(matched.rm_so)
                            .expect("regex match start is nonnegative");
                        let end =
                            usize::try_from(matched.rm_eo).expect("regex match end is nonnegative");
                        regsub_copy(buf, text, start, end);
                        index += 2;
                        continue;
                    }
                }
            }
            buf.push(escaped);
            index += 2;
        } else {
            buf.push(with[index]);
            index += 1;
        }
    }
}
pub fn regsub_cstring(
    pattern: &CStr,
    with: &CStr,
    text: &CStr,
    flags: ::core::ffi::c_int,
) -> Option<CString> {
    unsafe { regsub_raw(pattern, with, text, flags) }
}

unsafe fn regsub_raw(
    pattern: &CStr,
    with: &CStr,
    text: &CStr,
    flags: ::core::ffi::c_int,
) -> Option<CString> {
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
    let text_bytes = text.to_bytes();
    let pattern_bytes = pattern.to_bytes();
    let replacement_bytes = with.to_bytes();
    let mut start = 0;
    let mut last = 0;
    let mut empty = false;
    let mut buf = Vec::<u8>::new();
    if text_bytes.is_empty() {
        return Some(CString::default());
    }
    if pattern_bytes.is_empty() {
        return Some(text.to_owned());
    }
    if regcomp(&raw mut r, pattern.as_ptr(), flags) != 0 as ::core::ffi::c_int {
        return None;
    }
    let regex_owner = CompiledRegex::new(&raw mut r);
    let end = text_bytes.len();
    while start <= end {
        if regexec(
            &raw mut r,
            text.as_ptr().add(start),
            m.len() as size_t,
            &raw mut m as *mut regmatch_t,
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            regsub_copy(&mut buf, text_bytes, start, end);
            break;
        } else {
            let match_start = usize::try_from(m[0].rm_so)
                .expect("successful regex match has a nonnegative start");
            let match_end =
                usize::try_from(m[0].rm_eo).expect("successful regex match has a nonnegative end");
            let absolute_match_start = start + match_start;
            regsub_copy(&mut buf, text_bytes, last, absolute_match_start);
            if pattern_bytes[0] == b'^' {
                regsub_expand(&mut buf, replacement_bytes, &text_bytes[start..], &m);
                last = start + match_end;
                regsub_copy(&mut buf, text_bytes, last, end);
                break;
            } else if empty || absolute_match_start != last || m[0].rm_so != m[0].rm_eo {
                regsub_expand(&mut buf, replacement_bytes, &text_bytes[start..], &m);
                last = start + match_end;
                start += match_end;
                empty = false;
            } else {
                last = start + match_end;
                start += match_end + 1;
                empty = true;
            }
        }
    }
    drop(regex_owner);
    Some(CString::new(buf).expect("regex substitution contains no NUL"))
}

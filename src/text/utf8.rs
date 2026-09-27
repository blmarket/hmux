use crate::src::compat::utf8proc::utf8proc_wcwidth;
use crate::src::compat::vis::vis;
use crate::src::ffi::libc::{__ctype_b_loc, strlen};
use crate::src::log::{fatalx, log_bytes, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::ctype::_ISalpha;
use crate::src::shared::grid::*;
use crate::src::shared::options::{options_array_item, options_entry};
use crate::src::shared::utf8::wchar_t;
use crate::src::shared::utf8::*;
use crate::src::shared::vis::VIS_DQ;
use crate::src::text::utf8_cache::{UTF8_ITEMS, UTF8_WIDTHS};
use crate::src::text::utf8_decode::{decode_utf8, DecodeResult};
use crate::src::text::utf8_width::parse_width_override;
use crate::src::tmux::global_options;
use std::ffi::{CStr, CString};

fn utf8_find_in_width_cache(wc: wchar_t) -> Option<u_int> {
    UTF8_WIDTHS
        .lock()
        .expect("UTF-8 width cache poisoned")
        .find(wc)
}

unsafe fn utf8_insert_width_cache(wc: wchar_t, width: u_int) {
    log_debug(format_args!(
        "Unicode width cache: {:08X}={}",
        wc as u_int, width
    ));
    UTF8_WIDTHS
        .lock()
        .expect("UTF-8 width cache poisoned")
        .insert(wc, width);
}
unsafe fn utf8_add_to_width_cache(input: &CStr) {
    if let Some(parsed) = parse_width_override(input) {
        for codepoint in parsed.codepoints {
            utf8_insert_width_cache(codepoint, parsed.width);
        }
    }
}
pub unsafe fn utf8_update_width_cache() {
    UTF8_WIDTHS
        .lock()
        .expect("UTF-8 width cache poisoned")
        .reset_defaults();
    let mut o: *mut options_entry;
    let mut a: *mut options_array_item;
    o = options_get(
        global_options,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        utf8_add_to_width_cache(CStr::from_ptr((*options_array_item_value(a)).string_ptr()));
        a = options_array_next(a);
    }
}
unsafe fn utf8_put_item(data: &[u8]) -> Option<u_int> {
    let (index, inserted) = UTF8_ITEMS
        .lock()
        .expect("UTF-8 item cache poisoned")
        .intern(data)?;
    log_debug(format_args!(
        "utf8_put_item: {} {} = {}",
        if inserted { "added" } else { "found" },
        log_bytes(data),
        index,
    ));
    Some(index)
}

pub unsafe fn utf8_from_data(ud: &utf8_data, uc: &mut utf8_char) -> utf8_state {
    if ud.width > 2 {
        fatalx(|out| write!(out, "invalid UTF-8 width: {}", ud.width));
    }
    let index = if ud.size as usize > ud.data.len() {
        None
    } else if ud.size <= 3 {
        Some((u32::from(ud.data[2]) << 16) | (u32::from(ud.data[1]) << 8) | u32::from(ud.data[0]))
    } else {
        utf8_put_item(&ud.data[..ud.size as usize])
    };
    if let Some(index) = index {
        *uc = (u32::from(ud.size) << 24) | ((u32::from(ud.width) + 1) << 29) | index;
        log_debug(format_args!(
            "utf8_from_data: ({} {} {}) -> {:08x}",
            ud.width,
            ud.size,
            log_bytes(&ud.data[..ud.size as usize]),
            *uc,
        ));
        return UTF8_DONE;
    }
    *uc = match ud.width {
        0 => 1 << 29,
        1 => (1 << 24) | (2 << 29) | 0x20,
        _ => (1 << 24) | (2 << 29) | 0x2020,
    };
    UTF8_ERROR
}

pub unsafe fn utf8_to_data(uc: utf8_char, ud: &mut utf8_data) {
    *ud = utf8_data::default();
    ud.have = ((uc >> 24) & 0x1f) as u_char;
    ud.size = ud.have;
    ud.width = (uc >> 29).wrapping_sub(1) as u_char;
    if ud.size <= 3 {
        ud.data[2] = (uc >> 16) as u_char;
        ud.data[1] = (uc >> 8) as u_char;
        ud.data[0] = uc as u_char;
    } else {
        let cache = UTF8_ITEMS.lock().expect("UTF-8 item cache poisoned");
        let data = &mut ud.data[..ud.size as usize];
        if let Some(item) = cache.get(uc & 0xffffff) {
            data.copy_from_slice(&item[..data.len()]);
        } else {
            data.fill(b' ');
        }
    }
    log_debug(format_args!(
        "utf8_to_data: {:08x} -> ({} {} {})",
        uc,
        ud.width,
        ud.size,
        log_bytes(&ud.data[..ud.size as usize]),
    ));
}
pub unsafe fn utf8_build_one(mut ch: u_char) -> utf8_char {
    return (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
        | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
            << 29 as ::core::ffi::c_int
        | ch as utf8_char;
}
pub fn utf8_set(ud: &mut utf8_data, ch: u_char) {
    *ud = utf8_data {
        data: [0; 32],
        have: 1,
        size: 1,
        width: 1,
    };
    ud.data[0] = ch;
}

/// Return a stack value so a caller may safely copy onto its own source.
pub fn utf8_copy(from: &utf8_data) -> utf8_data {
    let mut copied = *from;
    let size = (copied.size as usize).min(copied.data.len());
    copied.data[size..].fill(0);
    copied
}

unsafe fn utf8_width(ud: &utf8_data) -> Option<i32> {
    let mut wc = 0;
    if utf8_towc(ud, &mut wc) != UTF8_DONE {
        return None;
    }
    if let Some(cached) = utf8_find_in_width_cache(wc) {
        log_debug(format_args!(
            "cached width for {:08X} is {}",
            wc as u_int, cached
        ));
        return Some(cached as i32);
    }
    let width = utf8proc_wcwidth(wc);
    log_debug(format_args!(
        "utf8proc_wcwidth({:05X}) returned {}",
        wc as u_int, width
    ));
    (0..=0xff).contains(&width).then_some(width)
}

pub unsafe fn utf8_towc(ud: &utf8_data, wc: &mut wchar_t) -> utf8_state {
    let Some(bytes) = ud.data.get(..ud.size as usize) else {
        return UTF8_ERROR;
    };
    match decode_utf8(bytes) {
        DecodeResult::Complete { codepoint, .. } => {
            *wc = codepoint as wchar_t;
            log_debug(format_args!(
                "UTF-8 {} is U+{:06X}",
                log_bytes(bytes),
                *wc as u_int
            ));
            UTF8_DONE
        }
        _ => {
            log_debug(format_args!("UTF-8 {} is invalid", log_bytes(bytes)));
            UTF8_ERROR
        }
    }
}

pub fn utf8_has_whitespace(ud: &utf8_data) -> ::core::ffi::c_int {
    let Some(mut remaining) = ud.data.get(..ud.size as usize) else {
        return 0;
    };
    while !remaining.is_empty() {
        let DecodeResult::Complete { codepoint, len } = decode_utf8(remaining) else {
            return 0;
        };
        remaining = &remaining[len..];
        if matches!(
            codepoint,
            9 | 10
                | 11
                | 12
                | 13
                | 32
                | 133
                | 160
                | 5760
                | 8192
                | 8193
                | 8194
                | 8195
                | 8196
                | 8197
                | 8198
                | 8199
                | 8200
                | 8201
                | 8202
                | 8232
                | 8233
                | 8239
                | 8287
                | 12288
        ) {
            return 1;
        }
    }
    0
}

pub unsafe fn utf8_fromwc(wc: wchar_t, ud: &mut utf8_data) -> utf8_state {
    let size = match crate::src::ffi::utf8proc::encode_cell(wc, &mut ud.data) {
        Ok(size) => size,
        Err(error) => {
            log_debug(format_args!("UTF-8 {}, wctomb() {}", wc, error));
            crate::src::ffi::utf8proc::reset_wctomb();
            return UTF8_ERROR;
        }
    };
    if size == 0 {
        return UTF8_ERROR;
    }
    ud.have = size as u_char;
    ud.size = ud.have;
    if let Some(width) = utf8_width(ud) {
        ud.width = width as u_char;
        UTF8_DONE
    } else {
        UTF8_ERROR
    }
}

pub unsafe fn utf8_open(ud: &mut utf8_data, ch: u_char) -> utf8_state {
    *ud = utf8_data::default();
    let DecodeResult::Incomplete { expected } = decode_utf8(&[ch]) else {
        return UTF8_ERROR;
    };
    if !(2..=4).contains(&expected) {
        return UTF8_ERROR;
    }
    ud.size = expected as u_char;
    utf8_append(ud, ch);
    UTF8_MORE
}

pub unsafe fn utf8_append(ud: &mut utf8_data, ch: u_char) -> utf8_state {
    if ud.have >= ud.size {
        fatalx(|out| out.write_all(b"UTF-8 character overflow"));
    }
    if ud.size as usize > ud.data.len() {
        fatalx(|out| out.write_all(b"UTF-8 character size too large"));
    }
    if ud.have != 0 && ch & 0xc0 != 0x80 {
        ud.width = 0xff;
    }
    ud.data[ud.have as usize] = ch;
    ud.have += 1;
    match decode_utf8(&ud.data[..ud.have as usize]) {
        DecodeResult::Complete { len, .. } if ud.have == ud.size && len == ud.size as usize => {
            if ud.width == 0xff {
                return UTF8_ERROR;
            }
            let Some(width) = utf8_width(ud) else {
                return UTF8_ERROR;
            };
            ud.width = width as u_char;
            UTF8_DONE
        }
        DecodeResult::Invalid { .. } if ud.have == ud.size => UTF8_ERROR,
        _ => UTF8_MORE,
    }
}
pub unsafe fn utf8_strvis(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut len: size_t,
    mut flag: ::core::ffi::c_int,
) -> size_t {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut start: *const ::core::ffi::c_char = dst;
    let mut end: *const ::core::ffi::c_char = src.offset(len as isize);
    let mut more: utf8_state = UTF8_MORE;
    let mut i: size_t = 0;
    while src < end {
        more = utf8_open(&mut ud, *src as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                src = src.offset(1);
                if !(src < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&mut ud, *src as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                i = 0 as size_t;
                while i < ud.size as size_t {
                    let fresh3 = dst;
                    dst = dst.offset(1);
                    *fresh3 = ud.data[i as usize] as ::core::ffi::c_char;
                    i = i.wrapping_add(1);
                }
                continue;
            } else {
                src = src.offset(-(ud.have as ::core::ffi::c_int as isize));
            }
        }
        if flag & VIS_DQ != 0
            && *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '$' as i32
            && src < end.offset(-(1 as ::core::ffi::c_int as isize))
        {
            if *(*__ctype_b_loc()).offset(*src.offset(1 as ::core::ffi::c_int as isize) as u_char
                as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '_' as i32
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
            {
                let fresh4 = dst;
                dst = dst.offset(1);
                *fresh4 = '\\' as i32 as ::core::ffi::c_char;
            }
            let fresh5 = dst;
            dst = dst.offset(1);
            *fresh5 = '$' as i32 as ::core::ffi::c_char;
        } else if src < end.offset(-(1 as ::core::ffi::c_int as isize)) {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        } else if src < end {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                '\0' as i32,
            );
        }
        src = src.offset(1);
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst.offset_from(start) as ::core::ffi::c_long as size_t;
}
/// Escape a C string using the same byte conversion as `utf8_strvis`, with
/// the result owned by Rust.
pub(crate) fn utf8_stravis_cstring(src: &CStr, flag: i32) -> CString {
    let source_len = src.to_bytes().len();
    // `utf8_strvis` writes at most four bytes for each source byte, followed
    // by one NUL. It initializes the buffer through that final NUL.
    let capacity = source_len
        .checked_mul(4)
        .and_then(|size| size.checked_add(1))
        .expect("escaped UTF-8 string is too large");
    let mut buffer = vec![0u8; capacity];
    let escaped_len =
        unsafe { utf8_strvis(buffer.as_mut_ptr().cast(), src.as_ptr(), source_len, flag) };
    buffer.truncate(escaped_len + 1);
    CString::from_vec_with_nul(buffer).expect("utf8_strvis output has no interior NUL")
}

/// Escape an explicit byte length without losing bytes after an input NUL.
pub(crate) fn utf8_stravisx_bytes(src: &[u8], flag: ::core::ffi::c_int) -> Vec<u8> {
    if src.is_empty() {
        return Vec::new();
    }
    let capacity = src
        .len()
        .checked_mul(4)
        .and_then(|size| size.checked_add(1))
        .expect("escaped UTF-8 bytes are too large");
    let mut buffer = vec![0u8; capacity];
    let escaped_len = unsafe {
        utf8_strvis(
            buffer.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            src.len(),
            flag,
        )
    };
    buffer.truncate(escaped_len);
    buffer
}
pub unsafe fn utf8_isvalid(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut more: utf8_state = UTF8_MORE;
    end = s.offset(strlen(s) as isize);
    while s < end {
        more = utf8_open(&mut ud, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(s < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&mut ud, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                continue;
            }
            return 0 as ::core::ffi::c_int;
        } else {
            if (*s as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
                || *s as ::core::ffi::c_int > 0x7e as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
            s = s.offset(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
/// The input is a NUL-terminated C string. The result is ASCII and retains
/// the old sanitizer's first-NUL view and underscore width for UTF-8 cells.
pub(crate) fn utf8_sanitize_cstring(src: &CStr) -> CString {
    let mut dst = Vec::new();
    let mut more: utf8_state = UTF8_MORE;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let source = src.to_bytes();
    let mut offset = 0;
    while offset < source.len() {
        let candidate_start = offset;
        more = unsafe { utf8_open(&mut ud, source[offset]) };
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                offset += 1;
                if offset >= source.len() || more != UTF8_MORE {
                    break;
                }
                more = unsafe { utf8_append(&mut ud, source[offset]) };
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                // xreallocarray rejected the old zero-sized request when a
                // leading zero-width UTF-8 cell had produced no bytes yet.
                if dst.is_empty() && ud.width == 0 {
                    unsafe { fatalx(|out| out.write_all(b"xreallocarray: zero size")) };
                }
                dst.resize(dst.len() + ud.width as usize, b'_');
                continue;
            } else {
                // Retry each byte after an invalid or truncated candidate.
                // Rewinding by `have` could step before the input for a
                // complete-length invalid sequence.
                offset = candidate_start;
            }
        }
        if source[offset] > 0x1f && source[offset] < 0x7f {
            dst.push(source[offset]);
        } else {
            dst.push(b'_');
        }
        offset += 1;
    }
    CString::new(dst).expect("sanitized bytes contain no interior NUL")
}
pub unsafe fn utf8_strlen(mut s: *const utf8_data) -> size_t {
    let mut i: size_t = 0;
    i = 0 as size_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        i = i.wrapping_add(1);
    }
    return i;
}
pub unsafe fn utf8_strwidth(mut s: *const utf8_data, mut n: ssize_t) -> u_int {
    let mut i: ssize_t = 0;
    let mut width: u_int = 0 as u_int;
    i = 0 as ssize_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if n != -(1 as ::core::ffi::c_int) as ssize_t && n == i {
            break;
        }
        width = width.wrapping_add((*s.offset(i as isize)).width as u_int);
        i += 1;
    }
    return width;
}
// Decode into Rust-owned cells while retaining the size-zero terminator used
// by the existing UTF-8 routines that borrow this array as a C-style view.
pub(crate) fn utf8_fromcstr_vec(src: &CStr) -> Vec<utf8_data> {
    let bytes = src.to_bytes();
    let mut cells = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        let mut more = unsafe { utf8_open(&mut cell, bytes[index] as u_char) };
        if more == UTF8_MORE {
            loop {
                index += 1;
                if index == bytes.len() || more != UTF8_MORE {
                    break;
                }
                more = unsafe { utf8_append(&mut cell, bytes[index] as u_char) };
            }
            if more == UTF8_DONE {
                cells.push(cell);
                continue;
            }
            index -= cell.have as usize;
        }
        utf8_set(&mut cell, bytes[index] as u_char);
        cells.push(cell);
        index += 1;
    }
    cells.push(utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    });
    cells
}

/// Copy the C-visible part of a sentinel-terminated cell stream into Rust-owned storage.
/// C consumers stop at the first NUL, even when it occurs inside a cell.
pub(crate) unsafe fn utf8_tocstr_cstring(mut src: *const utf8_data) -> CString {
    let mut bytes = Vec::new();
    while (*src).size != 0 {
        let data = &(&(*src).data)[..(*src).size as usize];
        if let Some(end) = data.iter().position(|&byte| byte == 0) {
            bytes.extend_from_slice(&data[..end]);
            break;
        }
        bytes.extend_from_slice(data);
        src = src.add(1);
    }
    CString::new(bytes).expect("the first NUL ends the copied string")
}
pub unsafe fn utf8_cstrwidth(mut s: *const ::core::ffi::c_char) -> u_int {
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut width: u_int = 0;
    let mut more: utf8_state = UTF8_MORE;
    width = 0 as u_int;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        more = utf8_open(&mut tmp, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(*s as ::core::ffi::c_int != '\0' as i32
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&mut tmp, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                width = width.wrapping_add(tmp.width as u_int);
                continue;
            } else {
                s = s.offset(-(tmp.have as ::core::ffi::c_int as isize));
            }
        }
        if *s as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        {
            width = width.wrapping_add(1);
        }
        s = s.offset(1);
    }
    return width;
}
pub(crate) fn utf8_pad_cstring(s: &CStr, width: u_int, left: bool) -> CString {
    let bytes = s.to_bytes();
    let padding = width.saturating_sub(unsafe { utf8_cstrwidth(s.as_ptr()) }) as usize;
    let mut output = Vec::with_capacity(bytes.len() + padding);
    if left {
        output.resize(padding, b' ');
        output.extend_from_slice(bytes);
    } else {
        output.extend_from_slice(bytes);
        output.resize(bytes.len() + padding, b' ');
    }
    CString::new(output).expect("padded C string contains no NUL")
}
pub unsafe fn utf8_cstrhas(
    s: *const ::core::ffi::c_char,
    ud: *const utf8_data,
) -> ::core::ffi::c_int {
    utf8_cstrhas_impl(CStr::from_ptr(s), &*ud) as ::core::ffi::c_int
}
pub(crate) fn utf8_cstrhas_impl(s: &CStr, ud: &utf8_data) -> bool {
    let bytes = s.to_bytes();
    let mut offset = 0;
    let mut found = 0;
    while offset < bytes.len() {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        let mut more = unsafe { utf8_open(&mut cell, bytes[offset]) };
        if more == UTF8_MORE {
            let start = offset;
            offset += 1;
            while offset < bytes.len() && more == UTF8_MORE {
                more = unsafe { utf8_append(&mut cell, bytes[offset]) };
                offset += 1;
            }
            if more != UTF8_DONE {
                // utf8_fromcstr retries every byte after an incomplete or
                // invalid candidate, including bytes consumed by utf8_append.
                offset = start;
            }
        }
        if more != UTF8_DONE {
            utf8_set(&mut cell, bytes[offset]);
            offset += 1;
        }
        let matches = {
            cell.size == ud.size && cell.data[..cell.size as usize] == ud.data[..ud.size as usize]
        };
        if matches {
            found = 1;
        }
    }
    found != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_owns_printable_ascii_and_preserves_legacy_widths() {
        {
            for (input, expected) in [
                (&b"\0"[..], &b""[..]),
                (&b"A\x01 \x7fB\0"[..], &b"A_ _B"[..]),
                (&b"caf\xc3\xa9\0"[..], &b"caf_"[..]),
                (&b"\xe4\xb8\xad\0"[..], &b"__"[..]),
                (&b"\xff\0"[..], &b"_"[..]),
                (&b"\xe2\x82\0"[..], &b"__"[..]),
                (&b"\xe2(\xa1\0"[..], &b"_(_"[..]),
                (&b"A\0B\0"[..], &b"A"[..]),
            ] {
                let nul = input.iter().position(|&byte| byte == 0).unwrap();
                let input = CStr::from_bytes_with_nul(&input[..=nul]).unwrap();
                assert_eq!(utf8_sanitize_cstring(input).as_bytes(), expected);
            }
        }
    }

    #[test]
    fn owned_cell_decoder_retries_each_byte_after_a_bad_sequence() {
        let cells = utf8_fromcstr_vec(CStr::from_bytes_with_nul(b"\xe2(\xa1\0").unwrap());
        assert_eq!(cells.len(), 4);
        for (cell, byte) in cells[..3].iter().zip([0xe2, b'(', 0xa1]) {
            assert_eq!(cell.size, 1);
            assert_eq!(cell.data[0], byte);
        }
        assert_eq!(cells[3].size, 0);
    }

    #[test]
    fn utf8_tocstr_cstring_stops_at_the_first_nul_in_a_cell_or_between_cells() {
        unsafe {
            for parts in [
                vec![],
                vec![vec![b'a'], vec![0xe2, 0x82, 0xac], vec![0xff]],
                vec![vec![b'a', 0, b'b'], vec![b'c']],
                vec![vec![b'a'], vec![0], vec![b'b']],
            ] {
                let mut cells = Vec::new();
                for part in parts {
                    let mut cell = utf8_data::default();
                    cell.data[..part.len()].copy_from_slice(&part);
                    cell.size = part.len() as u_char;
                    cells.push(cell);
                }
                cells.push(utf8_data::default());

                let mut expected = Vec::new();
                for cell in &cells {
                    if cell.size == 0 {
                        break;
                    }
                    let bytes = &cell.data[..cell.size as usize];
                    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
                        expected.extend_from_slice(&bytes[..end]);
                        break;
                    }
                    expected.extend_from_slice(bytes);
                }
                assert_eq!(utf8_tocstr_cstring(cells.as_ptr()).as_bytes(), expected);
            }
        }
    }

    #[test]
    fn owned_vis_helpers_preserve_multibyte_and_explicit_length_bytes() {
        {
            let input = CString::new(&b"a\xc3\xa9\xff"[..]).unwrap();
            let escaped =
                utf8_stravis_cstring(input.as_c_str(), crate::src::shared::vis::VIS_OCTAL);
            assert_eq!(escaped.as_bytes(), b"a\xc3\xa9\\377");

            let bytes = utf8_stravisx_bytes(b"a\0b", crate::src::shared::vis::VIS_OCTAL);
            assert_eq!(bytes, b"a\\000b");
        }
    }

    #[test]
    fn packed_cells_preserve_inline_and_cached_bytes() {
        unsafe {
            for width in 0..=2 {
                for size in 0..=31 {
                    let mut original = utf8_data {
                        have: size,
                        size,
                        width,
                        ..utf8_data::default()
                    };
                    for (index, byte) in original.data[..size as usize].iter_mut().enumerate() {
                        *byte = (index as u8).wrapping_mul(31).wrapping_add(0x80);
                    }
                    let mut encoded = 0;
                    assert_eq!(utf8_from_data(&original, &mut encoded), UTF8_DONE);
                    let mut repeated = 0;
                    assert_eq!(utf8_from_data(&original, &mut repeated), UTF8_DONE);
                    assert_eq!(encoded, repeated);
                    let mut decoded = utf8_data::default();
                    utf8_to_data(encoded, &mut decoded);
                    assert_eq!(
                        (decoded.have, decoded.size, decoded.width),
                        (size, size, width)
                    );
                    assert_eq!(decoded.data, original.data);
                }
            }
            // Inline packing includes three bytes even when the cell size is smaller.
            let mut cell = utf8_data {
                size: 1,
                width: 1,
                ..utf8_data::default()
            };
            cell.data[..3].copy_from_slice(b"abc");
            let mut encoded = 0;
            assert_eq!(utf8_from_data(&cell, &mut encoded), UTF8_DONE);
            assert_eq!(encoded, (1 << 24) | (2 << 29) | 0x636261);
        }
    }

    #[test]
    fn packed_cells_preserve_fallback_and_size_boundary() {
        unsafe {
            for (width, expected) in [(0, 1 << 29), (1, 0x41000020), (2, 0x41002020)] {
                let cell = utf8_data {
                    size: 33,
                    width,
                    ..utf8_data::default()
                };
                let mut encoded = 0;
                assert_eq!(utf8_from_data(&cell, &mut encoded), UTF8_ERROR);
                assert_eq!(encoded, expected);
            }
            let cell = utf8_data {
                data: [0xab; 32],
                size: 32,
                width: 1,
                ..utf8_data::default()
            };
            let mut encoded = 0;
            assert_eq!(utf8_from_data(&cell, &mut encoded), UTF8_DONE);
            // Match tmux's 5-bit size field, including its size-32 overlap with width.
            assert_eq!(encoded & 0xff000000, 0x60000000);
            let mut decoded = utf8_data::default();
            utf8_to_data((4 << 24) | (3 << 29) | 0xfffffe, &mut decoded);
            assert_eq!((decoded.have, decoded.size, decoded.width), (4, 4, 2));
            assert_eq!(&decoded.data[..4], b"    ");
            assert!(decoded.data[4..].iter().all(|&byte| byte == 0));
        }
    }

    #[test]
    fn utf8_width_cache_parses_entries_and_ignores_invalid_ones() {
        unsafe {
            for entry in [
                &b"U+E010=2\0"[..],
                &b"U+E011-U+E013=0\0"[..],
                &b"\xee\x80\xa0=1\0"[..], // U+E020, as a UTF-8 character.
                &b"U+E040=1\0U+E041=2\0"[..], // Stop at the first NUL.
                &b"z=2\0"[..],            // A single ASCII cell also uses this path.
            ] {
                utf8_add_to_width_cache(CStr::from_bytes_until_nul(entry).unwrap());
            }

            for (codepoint, expected) in [
                (0xE010, 2),
                (0xE011, 0),
                (0xE012, 0),
                (0xE013, 0),
                (0xE020, 1),
                (0xE040, 1),
                ('z' as i32, 2),
            ] {
                assert_eq!(
                    utf8_find_in_width_cache(codepoint),
                    Some(expected),
                    "wrong width for U+{codepoint:04X}"
                );
            }

            for entry in [
                &b"U+E030\0"[..],          // No separator.
                &b"U+E030=3\0"[..],        // Width out of range.
                &b"U+E030-U+E02F=1\0"[..], // Reversed range.
                &b"ab=1\0"[..],            // More than one character.
                &b"=1\0"[..],              // No character.
                &b"\xee\x80\xa2z=1\0"[..], // Valid UTF-8 cell followed by ASCII.
                &b"\xc3(=1\0"[..],         // Invalid continuation retries the first byte.
                &b"\xe2\x82=1\0"[..],      // Incomplete UTF-8 retries the first byte.
                &b"\xff=1\0"[..],          // Invalid single byte.
                &b"\xc0\xaf=1\0"[..],      // Overlong sequence.
            ] {
                utf8_add_to_width_cache(CStr::from_bytes_until_nul(entry).unwrap());
            }
            for codepoint in [0xE030, 0xE041, 0xE022, 'a' as i32, 'b' as i32, '(' as i32] {
                assert!(
                    utf8_find_in_width_cache(codepoint).is_none(),
                    "unexpected U+{codepoint:04X}"
                );
            }

            for codepoint in [0xE010, 0xE011, 0xE012, 0xE013, 0xE020, 0xE040, 'z' as i32] {
                UTF8_WIDTHS.lock().unwrap().remove(codepoint);
            }
        }
    }
}

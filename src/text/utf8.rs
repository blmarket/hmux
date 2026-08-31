use crate::compat::strtonum;
use crate::compat::vis;
use crate::compat::{utf8proc_mbtowc, utf8proc_wctomb, utf8proc_wcwidth};
use crate::ffi::{
    __ctype_b_loc, __ctype_get_mb_cur_max, __errno_location, mbtowc, strncmp, strtoull, wctomb,
};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::tree::GlobalTree;
use crate::types::{size_t, ssize_t, u_char, u_int, wchar_t};
use ::core::ffi::CStr;
use ::std::ffi::CString;
/// One character as its bytes, with how many of them have arrived, how many
/// the encoding says there are, and how wide it shows.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct utf8_data {
    pub data: [u_char; 32],
    pub have: u_char,
    pub size: u_char,
    pub width: u_char,
}

/// A short character packed into one word, as an extended grid cell keeps it.
pub type utf8_char = u_int;

/// How far reading a character got: `UTF8_MORE`, `UTF8_DONE` or `UTF8_ERROR`.
pub type utf8_state = ::core::ffi::c_uint;

pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalpha: ctype_mask = 1024;
pub const UTF8_ERROR: utf8_state = 2;
pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;
/// The bytes of a character too long to fit in a `utf8_char`, keyed the way
/// the index and data trees order them: by length first, then by the bytes.
type utf8_stored = (u_char, [u8; 32]);
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const __WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX__;
pub const ULLONG_MAX: ::core::ffi::c_ulonglong = (__LONG_LONG_MAX__ as ::core::ffi::c_ulonglong)
    .wrapping_mul(2 as ::core::ffi::c_ulonglong)
    .wrapping_add(1 as ::core::ffi::c_ulonglong);
pub const WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const UTF8_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;

/// The index no character can have, one past the last the 24 index bits hold.
const UTF8_INDEX_END: u_int = 0xffffff + 1;

/// The widths tmux uses for the codepoints whose terminal width is not the one
/// `wcwidth` reports, which `utf8_update_width_cache` puts in the cache first.
const UTF8_DEFAULT_WIDTHS: [(wchar_t, u_int); 162] = [
    (0x261d, 2),
    (0x26f9, 2),
    (0x270a, 2),
    (0x270b, 2),
    (0x270c, 2),
    (0x270d, 2),
    (0x1f1e6, 1),
    (0x1f1e7, 1),
    (0x1f1e8, 1),
    (0x1f1e9, 1),
    (0x1f1ea, 1),
    (0x1f1eb, 1),
    (0x1f1ec, 1),
    (0x1f1ed, 1),
    (0x1f1ee, 1),
    (0x1f1ef, 1),
    (0x1f1f0, 1),
    (0x1f1f1, 1),
    (0x1f1f2, 1),
    (0x1f1f3, 1),
    (0x1f1f4, 1),
    (0x1f1f5, 1),
    (0x1f1f6, 1),
    (0x1f1f7, 1),
    (0x1f1f8, 1),
    (0x1f1f9, 1),
    (0x1f1fa, 1),
    (0x1f1fb, 1),
    (0x1f1fc, 1),
    (0x1f1fd, 1),
    (0x1f1fe, 1),
    (0x1f1ff, 1),
    (0x1f385, 2),
    (0x1f3c2, 2),
    (0x1f3c3, 2),
    (0x1f3c4, 2),
    (0x1f3c7, 2),
    (0x1f3ca, 2),
    (0x1f3cb, 2),
    (0x1f3cc, 2),
    (0x1f3fb, 2),
    (0x1f3fc, 2),
    (0x1f3fd, 2),
    (0x1f3fe, 2),
    (0x1f3ff, 2),
    (0x1f442, 2),
    (0x1f443, 2),
    (0x1f446, 2),
    (0x1f447, 2),
    (0x1f448, 2),
    (0x1f449, 2),
    (0x1f44a, 2),
    (0x1f44b, 2),
    (0x1f44c, 2),
    (0x1f44d, 2),
    (0x1f44e, 2),
    (0x1f44f, 2),
    (0x1f450, 2),
    (0x1f466, 2),
    (0x1f467, 2),
    (0x1f468, 2),
    (0x1f469, 2),
    (0x1f46b, 2),
    (0x1f46c, 2),
    (0x1f46d, 2),
    (0x1f46e, 2),
    (0x1f470, 2),
    (0x1f471, 2),
    (0x1f472, 2),
    (0x1f473, 2),
    (0x1f474, 2),
    (0x1f475, 2),
    (0x1f476, 2),
    (0x1f477, 2),
    (0x1f478, 2),
    (0x1f47c, 2),
    (0x1f481, 2),
    (0x1f482, 2),
    (0x1f483, 2),
    (0x1f485, 2),
    (0x1f486, 2),
    (0x1f487, 2),
    (0x1f48f, 2),
    (0x1f491, 2),
    (0x1f4aa, 2),
    (0x1f574, 2),
    (0x1f575, 2),
    (0x1f57a, 2),
    (0x1f590, 2),
    (0x1f595, 2),
    (0x1f596, 2),
    (0x1f645, 2),
    (0x1f646, 2),
    (0x1f647, 2),
    (0x1f64b, 2),
    (0x1f64c, 2),
    (0x1f64d, 2),
    (0x1f64e, 2),
    (0x1f64f, 2),
    (0x1f6a3, 2),
    (0x1f6b4, 2),
    (0x1f6b5, 2),
    (0x1f6b6, 2),
    (0x1f6c0, 2),
    (0x1f6cc, 2),
    (0x1f90c, 2),
    (0x1f90f, 2),
    (0x1f918, 2),
    (0x1f919, 2),
    (0x1f91a, 2),
    (0x1f91b, 2),
    (0x1f91c, 2),
    (0x1f91d, 2),
    (0x1f91e, 2),
    (0x1f91f, 2),
    (0x1f926, 2),
    (0x1f930, 2),
    (0x1f931, 2),
    (0x1f932, 2),
    (0x1f933, 2),
    (0x1f934, 2),
    (0x1f935, 2),
    (0x1f936, 2),
    (0x1f937, 2),
    (0x1f938, 2),
    (0x1f939, 2),
    (0x1f93d, 2),
    (0x1f93e, 2),
    (0x1f977, 2),
    (0x1f9b5, 2),
    (0x1f9b6, 2),
    (0x1f9b8, 2),
    (0x1f9b9, 2),
    (0x1f9bb, 2),
    (0x1f9cd, 2),
    (0x1f9ce, 2),
    (0x1f9cf, 2),
    (0x1f9d1, 2),
    (0x1f9d2, 2),
    (0x1f9d3, 2),
    (0x1f9d4, 2),
    (0x1f9d5, 2),
    (0x1f9d6, 2),
    (0x1f9d7, 2),
    (0x1f9d8, 2),
    (0x1f9d9, 2),
    (0x1f9da, 2),
    (0x1f9db, 2),
    (0x1f9dc, 2),
    (0x1f9dd, 2),
    (0x1fac3, 2),
    (0x1fac4, 2),
    (0x1fac5, 2),
    (0x1faf0, 2),
    (0x1faf1, 2),
    (0x1faf2, 2),
    (0x1faf3, 2),
    (0x1faf4, 2),
    (0x1faf5, 2),
    (0x1faf6, 2),
    (0x1faf7, 2),
    (0x1faf8, 2),
];

static utf8_width_cache: GlobalTree<wchar_t, u_int> = GlobalTree::new();
static mut utf8_no_width: ::core::ffi::c_int = 0;
static mut utf8_next_index: u_int = 0;

static utf8_data_tree: GlobalTree<utf8_stored, u_int> = GlobalTree::new();

static utf8_index_tree: GlobalTree<u_int, utf8_stored> = GlobalTree::new();

/// The bytes a character holds.
fn utf8_bytes(ud: &utf8_data) -> &[u8] {
    &ud.data[..ud.size as usize]
}

/// The character at the front of `bytes` and how many bytes it took, or `None`
/// when what is there does not finish a UTF-8 character. Reading it one byte
/// at a time is what tells the width cache about it.
fn utf8_take(bytes: &[u8]) -> Option<(utf8_data, usize)> {
    unsafe {
        let mut ud = utf8_data::default();
        if utf8_open(&mut ud, bytes[0]) != UTF8_MORE {
            return None;
        }
        let mut state = UTF8_MORE;
        for &b in &bytes[1..] {
            state = utf8_append(&mut ud, b);
            if state != UTF8_MORE {
                break;
            }
        }
        (state == UTF8_DONE).then_some((ud, ud.have as usize))
    }
}

/// Whether `b` is `isalpha` under the process's current locale, which is what
/// makes a `$` inside double quotes look like the start of a variable.
fn utf8_is_alpha(b: u8) -> bool {
    let class = unsafe { *(*__ctype_b_loc()).add(b as usize) } as ctype_mask;
    class & _ISalpha != 0
}

/// The bytes of `data` as the index and data trees key them. A character is
/// never longer than `UTF8_SIZE`, which is the width of the key, so the bytes
/// past the character's own stay zero.
fn utf8_stored_of(data: &[u8]) -> utf8_stored {
    let mut bytes: [u8; 32] = [0; 32];
    bytes[..data.len()].copy_from_slice(data);
    (data.len() as u_char, bytes)
}

/// The width the cache holds for `wc`, or `None` when it has none.
fn utf8_find_in_width_cache(wc: wchar_t) -> Option<u_int> {
    utf8_width_cache.map().get(&wc).copied()
}

fn utf8_insert_width_cache(wc: wchar_t, width: u_int) {
    unsafe {
        log_debug(
            c"Unicode width cache: %08X=%u".as_ptr(),
            fmt_args![wc as u_int, width],
        );
        utf8_width_cache.map().insert(wc, width);
    }
}

/// The codepoint a `U+xxxx` spelling stands for and where its digits ended.
/// `strtoull` reads the number, so the spellings taken are exactly C's.
unsafe fn utf8_parse_codepoint(
    s: *const ::core::ffi::c_char,
) -> Option<(wchar_t, *const ::core::ffi::c_char)> {
    unsafe {
        if strncmp(s, c"U+".as_ptr(), 2 as size_t) != 0 {
            return None;
        }
        let digits = s.add(2);
        let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *__errno_location() = 0;
        let n = strtoull(digits, &raw mut endptr, 16);
        if n == 0
            || n > WCHAR_MAX as ::core::ffi::c_ulonglong
            || *__errno_location() == ERANGE && n == ULLONG_MAX
        {
            return None;
        }
        Some((n as wchar_t, endptr))
    }
}

/// Reads one `codepoint=width` line of the `codepoint-widths` option, which
/// spells the codepoint either as `U+xxxx`, as a range of two of those, or as
/// the character itself.
unsafe fn utf8_add_to_width_cache(s: &CStr) {
    unsafe {
        let text = s.to_bytes();
        let Some(at) = text.iter().position(|&b| b == b'=') else {
            return;
        };
        let Ok(width) = strtonum(s.as_ptr().add(at + 1), 0, 2) else {
            return;
        };
        let width = width as u_int;
        let mut copy = text[..at].to_vec();
        copy.push(0);
        let spec = copy.as_ptr() as *const ::core::ffi::c_char;

        if strncmp(spec, c"U+".as_ptr(), 2 as size_t) == 0 {
            let Some((first, endptr)) = utf8_parse_codepoint(spec) else {
                return;
            };
            let last = if *endptr == b'-' as ::core::ffi::c_char {
                let Some((last, endptr)) = utf8_parse_codepoint(endptr.add(1)) else {
                    return;
                };
                if *endptr != 0 || last < first {
                    return;
                }
                last
            } else {
                if *endptr != 0 {
                    return;
                }
                first
            };
            for wc in first..=last {
                utf8_insert_width_cache(wc, width);
            }
            return;
        }

        utf8_no_width = 1;
        let ud = utf8_fromcstr(spec);
        utf8_no_width = 0;
        let one = ud.len() == 1;
        let mut wc: wchar_t = 0;
        let read = one
            && utf8proc_mbtowc(
                &raw mut wc,
                ud[0].data.as_ptr() as *const ::core::ffi::c_char,
                ud[0].size as size_t,
            ) > 0;
        if read {
            utf8_insert_width_cache(wc, width);
        }
    }
}

/// Rebuilds the width cache from the built-in defaults, then applies each
/// `codepoint-widths` spec the caller hands over, in order.
pub fn utf8_update_width_cache(specs: impl IntoIterator<Item = CString>) {
    unsafe {
        {
            let cache = utf8_width_cache.map();
            cache.clear();
            for &(wc, width) in UTF8_DEFAULT_WIDTHS.iter() {
                cache.insert(wc, width);
            }
        }
        for spec in specs {
            utf8_add_to_width_cache(&spec);
        }
    }
}

/// The index the trees keep a character under, adding it if it is new, or
/// `None` once every index has been handed out.
unsafe fn utf8_put_item(data: &[u8]) -> Option<u_int> {
    unsafe {
        let stored = utf8_stored_of(data);
        if let Some(&index) = utf8_data_tree.map().get(&stored) {
            log_debug(
                c"%s: found %.*s = %u".as_ptr(),
                fmt_args![
                    c"utf8_put_item".as_ptr(),
                    data.len() as ::core::ffi::c_int,
                    data.as_ptr(),
                    index
                ],
            );
            return Some(index);
        }
        if utf8_next_index == UTF8_INDEX_END {
            return None;
        }
        let index = utf8_next_index;
        utf8_next_index += 1;
        utf8_index_tree.map().insert(index, stored);
        utf8_data_tree.map().insert(stored, index);
        log_debug(
            c"%s: added %.*s = %u".as_ptr(),
            fmt_args![
                c"utf8_put_item".as_ptr(),
                data.len() as ::core::ffi::c_int,
                data.as_ptr(),
                index
            ],
        );
        Some(index)
    }
}

/// Packs `ud` into a [`utf8_char`], along with whether the pack succeeded.
///
/// The character comes back either way: on failure it is the placeholder the
/// C wrote through its out-parameter, one space per column of `ud`'s width.
pub unsafe fn utf8_from_data(ud: &utf8_data) -> (utf8_state, utf8_char) {
    unsafe {
        if ud.width > 2 {
            fatalx(
                c"invalid UTF-8 width: %u".as_ptr(),
                fmt_args![ud.width as ::core::ffi::c_int],
            );
        }
        let index = if ud.size as ::core::ffi::c_int > UTF8_SIZE {
            None
        } else if ud.size <= 3 {
            Some(
                (ud.data[2] as utf8_char) << 16
                    | (ud.data[1] as utf8_char) << 8
                    | ud.data[0] as utf8_char,
            )
        } else {
            utf8_put_item(utf8_bytes(ud))
        };
        if let Some(index) = index {
            let uc = (ud.size as utf8_char) << 24 | (ud.width as utf8_char + 1) << 29 | index;
            log_debug(
                c"%s: (%d %d %.*s) -> %08x".as_ptr(),
                fmt_args![
                    c"utf8_from_data".as_ptr(),
                    ud.width as ::core::ffi::c_int,
                    ud.size as ::core::ffi::c_int,
                    ud.size as ::core::ffi::c_int,
                    &raw const ud.data as *const u_char,
                    uc
                ],
            );
            return (UTF8_DONE, uc);
        }
        let uc = match ud.width {
            0 => 1 << 29,
            1 => 1 << 24 | 2 << 29 | 0x20,
            _ => 1 << 24 | 2 << 29 | 0x2020,
        };
        (UTF8_ERROR, uc)
    }
}

pub fn utf8_to_data(uc: utf8_char, ud: &mut utf8_data) {
    unsafe {
        *ud = utf8_data::default();
        ud.have = (uc >> 24 & 0x1f) as u_char;
        ud.size = ud.have;
        ud.width = (uc >> 29).wrapping_sub(1) as u_char;
        if ud.size <= 3 {
            ud.data[2] = (uc >> 16) as u_char;
            ud.data[1] = (uc >> 8 & 0xff) as u_char;
            ud.data[0] = (uc & 0xff) as u_char;
        } else {
            let size = ud.size as usize;
            let stored = utf8_index_tree
                .map()
                .get(&((uc & 0xffffff) as u_int))
                .copied();
            let data = &mut (&mut ud.data)[..size];
            match stored {
                None => data.fill(b' '),
                Some((_, bytes)) => {
                    for (to, &from) in data.iter_mut().zip(bytes.iter()) {
                        *to = from;
                    }
                }
            }
        }
        log_debug(
            c"%s: %08x -> (%d %d %.*s)".as_ptr(),
            fmt_args![
                c"utf8_to_data".as_ptr(),
                uc,
                ud.width as ::core::ffi::c_int,
                ud.size as ::core::ffi::c_int,
                ud.size as ::core::ffi::c_int,
                &raw mut ud.data as *mut u_char
            ],
        );
    }
}

pub fn utf8_build_one(ch: u_char) -> utf8_char {
    1 << 24 | 2 << 29 | ch as utf8_char
}

pub fn utf8_set(ud: &mut utf8_data, ch: u_char) {
    *ud = utf8_data::default();
    ud.data[0] = ch;
    ud.have = 1;
    ud.size = 1;
    ud.width = 1;
}

pub fn utf8_copy(to: &mut utf8_data, from: &utf8_data) {
    *to = *from;
    let size = to.size as usize;
    to.data[size..].fill(0);
}

/// How wide a character is: what the width cache says, or what utf8proc says
/// when the cache has nothing for it.
unsafe fn utf8_width(ud: &utf8_data) -> Result<::core::ffi::c_int, utf8_state> {
    unsafe {
        let Some(wc) = utf8_towc(ud) else {
            return Err(UTF8_ERROR);
        };
        if let Some(cached) = utf8_find_in_width_cache(wc) {
            let width = cached as ::core::ffi::c_int;
            log_debug(
                c"cached width for %08X is %d".as_ptr(),
                fmt_args![wc as u_int, width],
            );
            return Ok(width);
        }
        let width = utf8proc_wcwidth(wc);
        log_debug(
            c"utf8proc_wcwidth(%05X) returned %d".as_ptr(),
            fmt_args![wc as u_int, width],
        );
        if !(0..=0xff).contains(&width) {
            return Err(UTF8_ERROR);
        }
        Ok(width)
    }
}

/// The codepoint a character stands for, as nothing when its bytes are not
/// one. `utf8proc_mbtowc` answers zero only for a null pointer, which the
/// character's own bytes never are, so the transpiled check for that is gone.
pub unsafe fn utf8_towc(ud: &utf8_data) -> Option<wchar_t> {
    unsafe {
        let mut wc: wchar_t = 0;
        if utf8proc_mbtowc(
            &raw mut wc,
            &raw const ud.data as *const u_char as *const ::core::ffi::c_char,
            ud.size as size_t,
        ) == -1
        {
            log_debug(
                c"UTF-8 %.*s, mbtowc() %d".as_ptr(),
                fmt_args![
                    ud.size as ::core::ffi::c_int,
                    &raw const ud.data as *const u_char,
                    *__errno_location()
                ],
            );
            mbtowc(
                ::core::ptr::null_mut::<wchar_t>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                __ctype_get_mb_cur_max(),
            );
            return None;
        }
        log_debug(
            c"UTF-8 %.*s is U+%06X".as_ptr(),
            fmt_args![
                ud.size as ::core::ffi::c_int,
                &raw const ud.data as *const u_char,
                wc as u_int
            ],
        );
        Some(wc)
    }
}

/// The character a codepoint is written as. `utf8proc_wctomb` writes nothing
/// only for a codepoint it has already turned down, so the transpiled check
/// for an empty answer is gone.
pub unsafe fn utf8_fromwc(wc: wchar_t, ud: &mut utf8_data) -> utf8_state {
    unsafe {
        let size = utf8proc_wctomb(
            &raw mut ud.data as *mut u_char as *mut ::core::ffi::c_char,
            wc,
        );
        if size < 0 {
            log_debug(
                c"UTF-8 %d, wctomb() %d".as_ptr(),
                fmt_args![wc, *__errno_location()],
            );
            wctomb(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as wchar_t);
            return UTF8_ERROR;
        }
        ud.have = size as u_char;
        ud.size = ud.have;
        let Ok(width) = utf8_width(ud) else {
            return UTF8_ERROR;
        };
        ud.width = width as u_char;
        UTF8_DONE
    }
}

pub fn utf8_open(ud: &mut utf8_data, ch: u_char) -> utf8_state {
    unsafe {
        *ud = utf8_data::default();
        ud.size = match ch {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => return UTF8_ERROR,
        };
        utf8_append(ud, ch);
        UTF8_MORE
    }
}

pub unsafe fn utf8_append(ud: &mut utf8_data, ch: u_char) -> utf8_state {
    unsafe {
        if ud.have >= ud.size {
            fatalx(c"UTF-8 character overflow".as_ptr(), fmt_args![]);
        }
        if ud.size as usize > ud.data.len() {
            fatalx(c"UTF-8 character size too large".as_ptr(), fmt_args![]);
        }
        if ud.have != 0 && ch & 0xc0 != 0x80 {
            ud.width = 0xff;
        }
        ud.data[ud.have as usize] = ch;
        ud.have += 1;
        if ud.have != ud.size {
            return UTF8_MORE;
        }
        if utf8_no_width != 0 {
            return UTF8_DONE;
        }
        if ud.width == 0xff {
            return UTF8_ERROR;
        }
        let Ok(width) = utf8_width(ud) else {
            return UTF8_ERROR;
        };
        ud.width = width as u_char;
        UTF8_DONE
    }
}

pub unsafe fn utf8_strvis(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    len: size_t,
    flag: ::core::ffi::c_int,
) -> size_t {
    unsafe {
        let bytes = ::core::slice::from_raw_parts(src as *const u_char, len);
        let mut out = dst;
        let mut i = 0;
        while i < bytes.len() {
            if let Some((ud, taken)) = utf8_take(&bytes[i..]) {
                for &b in utf8_bytes(&ud) {
                    *out = b as ::core::ffi::c_char;
                    out = out.add(1);
                }
                i += taken;
                continue;
            }
            let next = bytes.get(i + 1).copied();
            if flag & VIS_DQ != 0
                && bytes[i] == b'$'
                && let Some(next) = next
            {
                if utf8_is_alpha(next) || next == b'_' || next == b'{' {
                    *out = b'\\' as ::core::ffi::c_char;
                    out = out.add(1);
                }
                *out = b'$' as ::core::ffi::c_char;
                out = out.add(1);
            } else {
                out = vis(
                    out,
                    bytes[i] as ::core::ffi::c_int,
                    flag,
                    next.unwrap_or(0) as ::core::ffi::c_int,
                );
            }
            i += 1;
        }
        *out = 0;
        out.offset_from(dst) as size_t
    }
}

/// The visible form of `src` up to its terminator.
pub fn utf8_stravis(src: &CStr, flag: ::core::ffi::c_int) -> CString {
    utf8_stravisx(src.to_bytes(), flag)
}

/// The visible form of `src`, which no escape leaves a NUL in however the
/// source read.
pub fn utf8_stravisx(src: &[u8], flag: ::core::ffi::c_int) -> CString {
    unsafe {
        let mut buf: Vec<::core::ffi::c_char> = vec![0; 4 * (src.len() + 1)];
        let len = utf8_strvis(
            buf.as_mut_ptr(),
            src.as_ptr() as *const ::core::ffi::c_char,
            src.len() as size_t,
            flag,
        );
        CString::from_vec_unchecked(
            ::core::slice::from_raw_parts(buf.as_ptr() as *const u8, len as usize).to_vec(),
        )
    }
}

pub unsafe fn utf8_isvalid(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let bytes = ::core::ffi::CStr::from_ptr(s).to_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match utf8_take(&bytes[i..]) {
                Some((_, taken)) => i += taken,
                None => {
                    if !(0x20..=0x7e).contains(&bytes[i]) {
                        return 0;
                    }
                    i += 1;
                }
            }
        }
        1
    }
}

pub unsafe fn utf8_sanitize(src: *const ::core::ffi::c_char) -> CString {
    unsafe {
        let bytes = ::core::ffi::CStr::from_ptr(src).to_bytes();
        let mut out: Vec<u8> = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            match utf8_take(&bytes[i..]) {
                Some((ud, taken)) => {
                    out.resize(out.len() + ud.width as usize, b'_');
                    i += taken;
                }
                None => {
                    let b = bytes[i];
                    out.push(if (0x20..0x7f).contains(&b) { b } else { b'_' });
                    i += 1;
                }
            }
        }
        CString::new(out).expect("sanitized utf8 cannot contain NUL")
    }
}

/// How many characters an owned buffer holds, which holds every character in
/// it and nothing else: no terminator to stop at, and an empty one holds
/// nothing.
pub fn utf8_vec_strlen(s: &[utf8_data]) -> size_t {
    s.len() as size_t
}

/// The width of the first `n` characters of an owned buffer, or of all of
/// them when `n` is -1.
pub fn utf8_vec_strwidth(s: &[utf8_data], n: ssize_t) -> u_int {
    let take = if n == -1 { usize::MAX } else { n as usize };
    s.iter().take(take).map(|ud| ud.width as u_int).sum()
}

/// The bytes of an owned buffer, as one C string.
pub fn utf8_vec_tocstr(s: &[utf8_data]) -> CString {
    let mut out: Vec<u8> = Vec::new();
    for ud in s {
        out.extend_from_slice(utf8_bytes(ud));
    }
    CString::new(out).expect("utf8 bytes cannot contain NUL")
}

/// [`utf8_fromcstr`] as a buffer that owns what it holds.
pub unsafe fn utf8_vec_fromcstr(src: *const ::core::ffi::c_char) -> Vec<utf8_data> {
    unsafe {
        let bytes = ::core::ffi::CStr::from_ptr(src).to_bytes();
        let mut out: Vec<utf8_data> = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            match utf8_take(&bytes[i..]) {
                Some((ud, taken)) => {
                    out.push(ud);
                    i += taken;
                }
                None => {
                    let mut ud = utf8_data::default();
                    utf8_set(&mut ud, bytes[i]);
                    out.push(ud);
                    i += 1;
                }
            }
        }
        out
    }
}

pub unsafe fn utf8_fromcstr(src: *const ::core::ffi::c_char) -> Vec<utf8_data> {
    unsafe { utf8_vec_fromcstr(src) }
}

pub unsafe fn utf8_cstrwidth(s: *const ::core::ffi::c_char) -> u_int {
    unsafe {
        let bytes = ::core::ffi::CStr::from_ptr(s).to_bytes();
        let mut width: u_int = 0;
        let mut i = 0;
        while i < bytes.len() {
            match utf8_take(&bytes[i..]) {
                Some((ud, taken)) => {
                    width += ud.width as u_int;
                    i += taken;
                }
                None => {
                    if (0x20..=0x7e).contains(&bytes[i]) {
                        width += 1;
                    }
                    i += 1;
                }
            }
        }
        width
    }
}

/// `s` padded on the right with spaces to `width` display columns, or a plain
/// copy of it when it already fills them.
pub fn utf8_padcstr(s: &CStr, width: u_int) -> CString {
    let bytes = s.to_bytes();
    let n = unsafe { utf8_cstrwidth(s.as_ptr()) };
    let mut out = bytes.to_vec();
    if n < width {
        out.resize(bytes.len() + (width - n) as usize, b' ');
    }
    CString::new(out).expect("padding a C string cannot introduce NUL")
}

/// `s` padded on the left with spaces to `width` display columns, or a plain
/// copy of it when it already fills them.
pub fn utf8_rpadcstr(s: &CStr, width: u_int) -> CString {
    let bytes = s.to_bytes();
    let n = unsafe { utf8_cstrwidth(s.as_ptr()) };
    if n >= width {
        return s.to_owned();
    }
    let mut out = vec![b' '; (width - n) as usize];
    out.extend_from_slice(bytes);
    CString::new(out).expect("padding a C string cannot introduce NUL")
}

pub unsafe fn utf8_cstrhas(s: *const ::core::ffi::c_char, ud: &utf8_data) -> ::core::ffi::c_int {
    unsafe {
        let copy = utf8_fromcstr(s);
        let found = copy.iter().any(|one| utf8_bytes(one) == utf8_bytes(ud));
        found as ::core::ffi::c_int
    }
}

pub const __LONG_LONG_MAX__: ::core::ffi::c_longlong =
    9223372036854775807 as ::core::ffi::c_longlong;
pub const __WCHAR_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

#[cfg(test)]
#[path = "../tests/test_utf8.rs"]
mod tests;

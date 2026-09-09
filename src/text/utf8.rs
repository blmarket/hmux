use crate::compat::strtonum;
use crate::compat::vis;
use crate::compat::{utf8proc_mbtowc, utf8proc_wctomb, utf8proc_wcwidth};
use crate::ffi::{__ctype_get_mb_cur_max, __errno_location, mbtowc, strtoull, wctomb};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
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
pub type utf8_state = core::ffi::c_uint;

/// UTF-8 validation, display, and visual escaping without exposing codec state.
pub trait Utf8VisModel {
    /// Returns whether every character is printable and valid UTF-8.
    fn is_valid(&self, input: &CStr) -> bool;
    /// Replaces invalid or nonprintable input with display-width-preserving underscores.
    fn sanitize(&self, input: &CStr) -> CString;
    /// Returns the display width of a C string.
    fn width(&self, input: &CStr) -> u_int;
    /// Returns how many decoded characters a C string contains.
    fn character_count(&self, input: &CStr) -> usize;
    /// Returns the display width of the decoded characters.
    fn character_width(&self, input: &CStr) -> u_int;
    /// Decodes and re-encodes a C string.
    fn roundtrip(&self, input: &CStr) -> CString;
    /// Pads a C string on the right to a display width.
    fn pad_right(&self, input: &CStr, width: u_int) -> CString;
    /// Pads a C string on the left to a display width.
    fn pad_left(&self, input: &CStr, width: u_int) -> CString;
    /// Returns whether the first decoded character occurs in the input.
    fn contains_first_character(&self, input: &CStr) -> bool;
    /// Applies bytewise visual escaping.
    fn encode_bytes(&self, input: &[u8], flags: core::ffi::c_int) -> Vec<u8>;
    /// Applies visual escaping while preserving valid UTF-8.
    fn encode_utf8(&self, input: &[u8], flags: core::ffi::c_int) -> CString;
    /// Decodes visual escapes, including embedded NUL bytes.
    fn decode(&self, input: &CStr) -> Option<Vec<u8>>;
    /// Decodes visual escapes as a C string, stopping at the first NUL.
    fn decode_cstr(&self, input: &CStr) -> Option<CString> {
        let decoded = self.decode(input)?;
        let end = decoded
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(decoded.len());
        CString::new(&decoded[..end]).ok()
    }
}

/// The Rust UTF-8 and visual-escape implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustUtf8VisModel;

impl Utf8VisModel for RustUtf8VisModel {
    fn is_valid(&self, input: &CStr) -> bool {
        utf8_isvalid(input) != 0
    }

    fn sanitize(&self, input: &CStr) -> CString {
        utf8_sanitize(input)
    }

    fn width(&self, input: &CStr) -> u_int {
        utf8_cstrwidth(input)
    }

    fn character_count(&self, input: &CStr) -> usize {
        utf8_fromcstr(input).len()
    }

    fn character_width(&self, input: &CStr) -> u_int {
        utf8_vec_strwidth(&utf8_fromcstr(input), -1)
    }

    fn roundtrip(&self, input: &CStr) -> CString {
        utf8_vec_tocstr(&utf8_fromcstr(input))
    }

    fn pad_right(&self, input: &CStr, width: u_int) -> CString {
        utf8_padcstr(input, width)
    }

    fn pad_left(&self, input: &CStr, width: u_int) -> CString {
        utf8_rpadcstr(input, width)
    }

    fn contains_first_character(&self, input: &CStr) -> bool {
        utf8_fromcstr(input)
            .first()
            .is_some_and(|first| utf8_cstrhas(input, first) != 0)
    }

    fn encode_bytes(&self, input: &[u8], flags: core::ffi::c_int) -> Vec<u8> {
        crate::compat::strvisx(input, flags)
    }

    fn encode_utf8(&self, input: &[u8], flags: core::ffi::c_int) -> CString {
        utf8_stravisx(input, flags)
    }

    fn decode(&self, input: &CStr) -> Option<Vec<u8>> {
        let result = crate::compat::strnunvis(input, input.to_bytes().len() + 1);
        (result.status >= 0).then(|| result.output[..result.status as usize].to_vec())
    }
}

pub use crate::consts::{
    __LONG_LONG_MAX__, ERANGE, UTF8_DONE, UTF8_ERROR, UTF8_MORE, UTF8_SIZE, VIS_DQ,
};

/// The bytes of a character too long to fit in a `utf8_char`, keyed the way
/// the index and data trees order them: by length first, then by the bytes.
type utf8_stored = (u_char, [u8; 32]);

pub const __WCHAR_MAX: core::ffi::c_int = __WCHAR_MAX__;
pub const ULLONG_MAX: core::ffi::c_ulonglong = (__LONG_LONG_MAX__ as core::ffi::c_ulonglong)
    .wrapping_mul(2 as core::ffi::c_ulonglong)
    .wrapping_add(1 as core::ffi::c_ulonglong);
pub const WCHAR_MAX: core::ffi::c_int = __WCHAR_MAX;

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

thread_local! {
    static UTF8_WIDTH_CACHE: std::cell::RefCell<std::collections::BTreeMap<wchar_t, u_int>> = const {
        std::cell::RefCell::new(std::collections::BTreeMap::new())
    };
    static UTF8_NO_WIDTH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn without_width<R>(read: impl FnOnce() -> R) -> R {
    struct Restore<'a> {
        flag: &'a std::cell::Cell<bool>,
        previous: bool,
    }
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            self.flag.set(self.previous);
        }
    }
    UTF8_NO_WIDTH.with(|flag| {
        let _restore = Restore {
            flag,
            previous: flag.replace(true),
        };
        read()
    })
}
struct Utf8Store {
    next_index: u_int,
    by_data: std::collections::BTreeMap<utf8_stored, u_int>,
    by_index: std::collections::BTreeMap<u_int, utf8_stored>,
}

impl Utf8Store {
    const fn new() -> Self {
        Self {
            next_index: 0,
            by_data: std::collections::BTreeMap::new(),
            by_index: std::collections::BTreeMap::new(),
        }
    }

    fn intern(&mut self, stored: utf8_stored) -> Option<(u_int, bool)> {
        if let Some(&index) = self.by_data.get(&stored) {
            return Some((index, false));
        }
        if self.next_index == UTF8_INDEX_END {
            return None;
        }
        let index = self.next_index;
        self.next_index += 1;
        self.by_index.insert(index, stored);
        self.by_data.insert(stored, index);
        Some((index, true))
    }
}

static UTF8_STORE: std::sync::Mutex<Utf8Store> = std::sync::Mutex::new(Utf8Store::new());

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
    unsafe { libc::isalpha(b.into()) != 0 }
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
    UTF8_WIDTH_CACHE.with_borrow(|cache| cache.get(&wc).copied())
}

fn utf8_insert_width_cache(wc: wchar_t, width: u_int) {
    {
        log_debug(
            c"Unicode width cache: %08X=%u",
            fmt_args![wc as u_int, width],
        );
        UTF8_WIDTH_CACHE.with_borrow_mut(|cache| cache.insert(wc, width));
    }
}

/// The codepoint a `U+xxxx` spelling stands for and where its digits ended.
/// `strtoull` reads the number, so the spellings taken are exactly C's.
fn utf8_parse_codepoint(s: &CStr) -> Option<(wchar_t, &CStr)> {
    unsafe {
        if !s.to_bytes().starts_with(b"U+") {
            return None;
        }
        let digits = CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[2..])
            .expect("the codepoint digits retain their terminator");
        let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
        *__errno_location() = 0;
        let n = strtoull(digits.as_ptr(), &raw mut endptr, 16);
        if n == 0
            || n > WCHAR_MAX as core::ffi::c_ulonglong
            || *__errno_location() == ERANGE && n == ULLONG_MAX
        {
            return None;
        }
        let end = usize::try_from(endptr.offset_from(s.as_ptr())).ok()?;
        let rest = CStr::from_bytes_with_nul(s.to_bytes_with_nul().get(end..)?).ok()?;
        Some((n as wchar_t, rest))
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
        let width_text = CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[at + 1..])
            .expect("the width retains its terminator");
        let Ok(width) = strtonum(width_text, 0, 2) else {
            return;
        };
        let width = width as u_int;
        let mut copy = text[..at].to_vec();
        copy.push(0);
        let spec = CStr::from_bytes_with_nul(&copy).expect("the width specification is terminated");

        if spec.to_bytes().starts_with(b"U+") {
            let Some((first, rest)) = utf8_parse_codepoint(spec) else {
                return;
            };
            let last = if let Some(range) = rest.to_bytes_with_nul().strip_prefix(b"-") {
                let range =
                    CStr::from_bytes_with_nul(range).expect("the range retains its terminator");
                let Some((last, rest)) = utf8_parse_codepoint(range) else {
                    return;
                };
                if !rest.to_bytes().is_empty() || last < first {
                    return;
                }
                last
            } else {
                if !rest.to_bytes().is_empty() {
                    return;
                }
                first
            };
            for wc in first..=last {
                utf8_insert_width_cache(wc, width);
            }
            return;
        }

        let ud = without_width(|| utf8_fromcstr(spec));
        let one = ud.len() == 1;
        let mut wc: wchar_t = 0;
        let read =
            one && utf8proc_mbtowc(Some(&mut wc), Some(&ud[0].data[..ud[0].size as usize])) > 0;
        if read {
            utf8_insert_width_cache(wc, width);
        }
    }
}

/// Rebuilds the width cache from the built-in defaults, then applies each
/// `codepoint-widths` spec the caller hands over, in order.
pub fn utf8_update_width_cache(specs: impl IntoIterator<Item = CString>) {
    unsafe {
        UTF8_WIDTH_CACHE.with_borrow_mut(|cache| {
            cache.clear();
            for &(wc, width) in UTF8_DEFAULT_WIDTHS.iter() {
                cache.insert(wc, width);
            }
        });
        for spec in specs {
            utf8_add_to_width_cache(&spec);
        }
    }
}

/// The index the trees keep a character under, adding it if it is new, or
/// `None` once every index has been handed out.
unsafe fn utf8_put_item(data: &[u8]) -> Option<u_int> {
    {
        let stored = utf8_stored_of(data);
        let (index, inserted) = UTF8_STORE
            .lock()
            .expect("UTF-8 store lock is not poisoned")
            .intern(stored)?;
        log_debug(
            if inserted {
                c"%s: added %.*s = %u"
            } else {
                c"%s: found %.*s = %u"
            },
            fmt_args![
                c"utf8_put_item".as_ptr(),
                data.len() as core::ffi::c_int,
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
                c"invalid UTF-8 width: %u",
                fmt_args![ud.width as core::ffi::c_int],
            );
        }
        let index = if ud.size as core::ffi::c_int > UTF8_SIZE {
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
                c"%s: (%d %d %.*s) -> %08x",
                fmt_args![
                    c"utf8_from_data".as_ptr(),
                    ud.width as core::ffi::c_int,
                    ud.size as core::ffi::c_int,
                    ud.size as core::ffi::c_int,
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
    {
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
            let stored = UTF8_STORE
                .lock()
                .expect("UTF-8 store lock is not poisoned")
                .by_index
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
            c"%s: %08x -> (%d %d %.*s)",
            fmt_args![
                c"utf8_to_data".as_ptr(),
                uc,
                ud.width as core::ffi::c_int,
                ud.size as core::ffi::c_int,
                ud.size as core::ffi::c_int,
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
unsafe fn utf8_width(ud: &utf8_data) -> Result<core::ffi::c_int, utf8_state> {
    unsafe {
        let Some(wc) = utf8_towc(ud) else {
            return Err(UTF8_ERROR);
        };
        if let Some(cached) = utf8_find_in_width_cache(wc) {
            let width = cached as core::ffi::c_int;
            log_debug(
                c"cached width for %08X is %d",
                fmt_args![wc as u_int, width],
            );
            return Ok(width);
        }
        let width = utf8proc_wcwidth(wc);
        log_debug(
            c"utf8proc_wcwidth(%05X) returned %d",
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
        if utf8proc_mbtowc(Some(&mut wc), Some(&ud.data[..ud.size as usize])) == -1 {
            log_debug(
                c"UTF-8 %.*s, mbtowc() %d",
                fmt_args![
                    ud.size as core::ffi::c_int,
                    &raw const ud.data as *const u_char,
                    *__errno_location()
                ],
            );
            mbtowc(
                core::ptr::null_mut::<wchar_t>(),
                core::ptr::null::<core::ffi::c_char>(),
                __ctype_get_mb_cur_max(),
            );
            return None;
        }
        log_debug(
            c"UTF-8 %.*s is U+%06X",
            fmt_args![
                ud.size as core::ffi::c_int,
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
        let size = utf8proc_wctomb(Some((&mut ud.data[..4]).try_into().unwrap()), wc);
        if size < 0 {
            log_debug(c"UTF-8 %d, wctomb() %d", fmt_args![wc, *__errno_location()]);
            wctomb(core::ptr::null_mut::<core::ffi::c_char>(), 0 as wchar_t);
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
            fatalx(c"UTF-8 character overflow", fmt_args![]);
        }
        if ud.size as usize > ud.data.len() {
            fatalx(c"UTF-8 character size too large", fmt_args![]);
        }
        if ud.have != 0 && ch & 0xc0 != 0x80 {
            ud.width = 0xff;
        }
        ud.data[ud.have as usize] = ch;
        ud.have += 1;
        if ud.have != ud.size {
            return UTF8_MORE;
        }
        if UTF8_NO_WIDTH.get() {
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

fn utf8_strvis(bytes: &[u_char], flag: core::ffi::c_int) -> Vec<u8> {
    let mut output = Vec::with_capacity(4 * (bytes.len() + 1));
    let mut i = 0;
    while i < bytes.len() {
        if let Some((ud, taken)) = utf8_take(&bytes[i..]) {
            output.extend_from_slice(utf8_bytes(&ud));
            i += taken;
            continue;
        }
        let next = bytes.get(i + 1).copied();
        if flag & VIS_DQ != 0
            && bytes[i] == b'$'
            && let Some(next) = next
        {
            if utf8_is_alpha(next) || next == b'_' || next == b'{' {
                output.push(b'\\');
            }
            output.push(b'$');
        } else {
            output.extend_from_slice(&vis(
                bytes[i] as core::ffi::c_int,
                flag,
                next.unwrap_or(0) as core::ffi::c_int,
            ));
        }
        i += 1;
    }
    output
}

/// The visible form of `src`, which no escape leaves a NUL in however the
/// source read.
fn utf8_stravisx(src: &[u8], flag: core::ffi::c_int) -> CString {
    CString::new(utf8_strvis(src, flag)).expect("an encoded string holds no nul")
}

fn utf8_isvalid(s: &core::ffi::CStr) -> core::ffi::c_int {
    let bytes = s.to_bytes();
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

fn utf8_sanitize(src: &core::ffi::CStr) -> CString {
    let bytes = src.to_bytes();
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

pub fn utf8_fromcstr(src: &core::ffi::CStr) -> Vec<utf8_data> {
    let bytes = src.to_bytes();
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

fn utf8_cstrwidth(s: &core::ffi::CStr) -> u_int {
    let bytes = s.to_bytes();
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

/// `s` padded on the right with spaces to `width` display columns, or a plain
/// copy of it when it already fills them.
fn utf8_padcstr(s: &CStr, width: u_int) -> CString {
    let bytes = s.to_bytes();
    let n = utf8_cstrwidth(s);
    let mut out = bytes.to_vec();
    if n < width {
        out.resize(bytes.len() + (width - n) as usize, b' ');
    }
    CString::new(out).expect("padding a C string cannot introduce NUL")
}

/// `s` padded on the left with spaces to `width` display columns, or a plain
/// copy of it when it already fills them.
fn utf8_rpadcstr(s: &CStr, width: u_int) -> CString {
    let bytes = s.to_bytes();
    let n = utf8_cstrwidth(s);
    if n >= width {
        return s.to_owned();
    }
    let mut out = vec![b' '; (width - n) as usize];
    out.extend_from_slice(bytes);
    CString::new(out).expect("padding a C string cannot introduce NUL")
}

pub fn utf8_cstrhas(s: &core::ffi::CStr, ud: &utf8_data) -> core::ffi::c_int {
    let copy = utf8_fromcstr(s);
    let found = copy.iter().any(|one| utf8_bytes(one) == utf8_bytes(ud));
    found as core::ffi::c_int
}

pub const __WCHAR_MAX__: core::ffi::c_int = 2147483647 as core::ffi::c_int;

#[cfg(test)]
#[path = "../tests/test_utf8.rs"]
mod tests;

use crate::tty::tty_client;
use super::term::{tty_term_has, tty_term_number, tty_term_of};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};

pub const TTYC_U8: tty_code_code = 230;
pub const CLIENT_UTF8: c_int = 0x10000 as c_int;

/// The UTF-8 string an ACS key stands for. The table below is sorted by key.
struct tty_acs_entry {
    key: u8,
    string: &'static CStr,
}

/// The ACS key a UTF-8 string stands for. The two tables below are sorted by
/// string, one per string length.
struct tty_acs_reverse_entry {
    string: &'static CStr,
    key: u8,
}

/// One border character as a single UTF-8 cell. The empty string is the cell
/// type that draws nothing.
const fn border(s: &str) -> utf8_data {
    let bytes = s.as_bytes();
    let mut data = [0 as u_char; 32];
    let mut i = 0;
    while i < bytes.len() {
        data[i] = bytes[i];
        i += 1;
    }
    utf8_data {
        data,
        have: 0,
        size: bytes.len() as u_char,
        width: if bytes.is_empty() { 0 } else { 1 },
    }
}

static tty_acs_table: [tty_acs_entry; 36] = [
    tty_acs_entry {
        key: b'+',
        string: c"\u{2192}",
    },
    tty_acs_entry {
        key: b',',
        string: c"\u{2190}",
    },
    tty_acs_entry {
        key: b'-',
        string: c"\u{2191}",
    },
    tty_acs_entry {
        key: b'.',
        string: c"\u{2193}",
    },
    tty_acs_entry {
        key: b'0',
        string: c"\u{25ae}",
    },
    tty_acs_entry {
        key: b'`',
        string: c"\u{25c6}",
    },
    tty_acs_entry {
        key: b'a',
        string: c"\u{2592}",
    },
    tty_acs_entry {
        key: b'b',
        string: c"\u{2409}",
    },
    tty_acs_entry {
        key: b'c',
        string: c"\u{240c}",
    },
    tty_acs_entry {
        key: b'd',
        string: c"\u{240d}",
    },
    tty_acs_entry {
        key: b'e',
        string: c"\u{240a}",
    },
    tty_acs_entry {
        key: b'f',
        string: c"\u{00b0}",
    },
    tty_acs_entry {
        key: b'g',
        string: c"\u{00b1}",
    },
    tty_acs_entry {
        key: b'h',
        string: c"\u{2424}",
    },
    tty_acs_entry {
        key: b'i',
        string: c"\u{240b}",
    },
    tty_acs_entry {
        key: b'j',
        string: c"\u{2518}",
    },
    tty_acs_entry {
        key: b'k',
        string: c"\u{2510}",
    },
    tty_acs_entry {
        key: b'l',
        string: c"\u{250c}",
    },
    tty_acs_entry {
        key: b'm',
        string: c"\u{2514}",
    },
    tty_acs_entry {
        key: b'n',
        string: c"\u{253c}",
    },
    tty_acs_entry {
        key: b'o',
        string: c"\u{23ba}",
    },
    tty_acs_entry {
        key: b'p',
        string: c"\u{23bb}",
    },
    tty_acs_entry {
        key: b'q',
        string: c"\u{2500}",
    },
    tty_acs_entry {
        key: b'r',
        string: c"\u{23bc}",
    },
    tty_acs_entry {
        key: b's',
        string: c"\u{23bd}",
    },
    tty_acs_entry {
        key: b't',
        string: c"\u{251c}",
    },
    tty_acs_entry {
        key: b'u',
        string: c"\u{2524}",
    },
    tty_acs_entry {
        key: b'v',
        string: c"\u{2534}",
    },
    tty_acs_entry {
        key: b'w',
        string: c"\u{252c}",
    },
    tty_acs_entry {
        key: b'x',
        string: c"\u{2502}",
    },
    tty_acs_entry {
        key: b'y',
        string: c"\u{2264}",
    },
    tty_acs_entry {
        key: b'z',
        string: c"\u{2265}",
    },
    tty_acs_entry {
        key: b'{',
        string: c"\u{03c0}",
    },
    tty_acs_entry {
        key: b'|',
        string: c"\u{2260}",
    },
    tty_acs_entry {
        key: b'}',
        string: c"\u{00a3}",
    },
    tty_acs_entry {
        key: b'~',
        string: c"\u{00b7}",
    },
];

static tty_acs_reverse2: [tty_acs_reverse_entry; 1] = [tty_acs_reverse_entry {
    string: c"\u{00b7}",
    key: b'~',
}];

static tty_acs_reverse3: [tty_acs_reverse_entry; 32] = [
    tty_acs_reverse_entry {
        string: c"\u{2500}",
        key: b'q',
    },
    tty_acs_reverse_entry {
        string: c"\u{2501}",
        key: b'q',
    },
    tty_acs_reverse_entry {
        string: c"\u{2502}",
        key: b'x',
    },
    tty_acs_reverse_entry {
        string: c"\u{2503}",
        key: b'x',
    },
    tty_acs_reverse_entry {
        string: c"\u{250c}",
        key: b'l',
    },
    tty_acs_reverse_entry {
        string: c"\u{250f}",
        key: b'k',
    },
    tty_acs_reverse_entry {
        string: c"\u{2510}",
        key: b'k',
    },
    tty_acs_reverse_entry {
        string: c"\u{2513}",
        key: b'l',
    },
    tty_acs_reverse_entry {
        string: c"\u{2514}",
        key: b'm',
    },
    tty_acs_reverse_entry {
        string: c"\u{2517}",
        key: b'm',
    },
    tty_acs_reverse_entry {
        string: c"\u{2518}",
        key: b'j',
    },
    tty_acs_reverse_entry {
        string: c"\u{251b}",
        key: b'j',
    },
    tty_acs_reverse_entry {
        string: c"\u{251c}",
        key: b't',
    },
    tty_acs_reverse_entry {
        string: c"\u{2523}",
        key: b't',
    },
    tty_acs_reverse_entry {
        string: c"\u{2524}",
        key: b'u',
    },
    tty_acs_reverse_entry {
        string: c"\u{252b}",
        key: b'u',
    },
    tty_acs_reverse_entry {
        string: c"\u{2533}",
        key: b'w',
    },
    tty_acs_reverse_entry {
        string: c"\u{2534}",
        key: b'v',
    },
    tty_acs_reverse_entry {
        string: c"\u{253b}",
        key: b'v',
    },
    tty_acs_reverse_entry {
        string: c"\u{253c}",
        key: b'n',
    },
    tty_acs_reverse_entry {
        string: c"\u{254b}",
        key: b'n',
    },
    tty_acs_reverse_entry {
        string: c"\u{2550}",
        key: b'q',
    },
    tty_acs_reverse_entry {
        string: c"\u{2551}",
        key: b'x',
    },
    tty_acs_reverse_entry {
        string: c"\u{2554}",
        key: b'l',
    },
    tty_acs_reverse_entry {
        string: c"\u{2557}",
        key: b'k',
    },
    tty_acs_reverse_entry {
        string: c"\u{255a}",
        key: b'm',
    },
    tty_acs_reverse_entry {
        string: c"\u{255d}",
        key: b'j',
    },
    tty_acs_reverse_entry {
        string: c"\u{2560}",
        key: b't',
    },
    tty_acs_reverse_entry {
        string: c"\u{2563}",
        key: b'u',
    },
    tty_acs_reverse_entry {
        string: c"\u{2566}",
        key: b'w',
    },
    tty_acs_reverse_entry {
        string: c"\u{2569}",
        key: b'v',
    },
    tty_acs_reverse_entry {
        string: c"\u{256c}",
        key: b'n',
    },
];

static tty_acs_double_borders_list: [utf8_data; 13] = [
    border(""),
    border("\u{2551}"),
    border("\u{2550}"),
    border("\u{2554}"),
    border("\u{2557}"),
    border("\u{255a}"),
    border("\u{255d}"),
    border("\u{2566}"),
    border("\u{2569}"),
    border("\u{2560}"),
    border("\u{2563}"),
    border("\u{256c}"),
    border("\u{00b7}"),
];

static tty_acs_heavy_borders_list: [utf8_data; 13] = [
    border(""),
    border("\u{2503}"),
    border("\u{2501}"),
    border("\u{250f}"),
    border("\u{2513}"),
    border("\u{2517}"),
    border("\u{251b}"),
    border("\u{2533}"),
    border("\u{253b}"),
    border("\u{2523}"),
    border("\u{252b}"),
    border("\u{254b}"),
    border("\u{00b7}"),
];

static tty_acs_rounded_borders_list: [utf8_data; 13] = [
    border(""),
    border("\u{2502}"),
    border("\u{2500}"),
    border("\u{256d}"),
    border("\u{256e}"),
    border("\u{2570}"),
    border("\u{256f}"),
    border("\u{2533}"),
    border("\u{253b}"),
    border("\u{251c}"),
    border("\u{2524}"),
    border("\u{254b}"),
    border("\u{00b7}"),
];

/// `cell_type` is one of the thirteen border cell types the drawing code
/// names, `CELL_INSIDE` through `CELL_OUTSIDE`; `CELL_SCROLLBAR` never reaches
/// here, since both callers turn it away first.
pub fn tty_acs_double_borders(cell_type: c_int) -> &'static utf8_data {
    &tty_acs_double_borders_list[cell_type as usize]
}

/// See [`tty_acs_double_borders`] for the range of `cell_type`.
pub fn tty_acs_heavy_borders(cell_type: c_int) -> &'static utf8_data {
    &tty_acs_heavy_borders_list[cell_type as usize]
}

/// See [`tty_acs_double_borders`] for the range of `cell_type`.
pub fn tty_acs_rounded_borders(cell_type: c_int) -> &'static utf8_data {
    &tty_acs_rounded_borders_list[cell_type as usize]
}

/// Whether the terminal wants the ACS character set rather than UTF-8.
///
/// A `U8` capability of zero marks a terminal that cannot do UTF-8 and ACS
/// together, which is how a user turns UTF-8 line drawing off; otherwise the
/// client's own UTF-8 flag decides.
pub unsafe fn tty_acs_needed(tty: *mut tty) -> c_int {
    unsafe {
        if tty.is_null() {
            return 0;
        }
        if tty_term_has(tty_term_of(&*tty), TTYC_U8) != 0
            && tty_term_number(tty_term_of(&*tty), TTYC_U8) == 0
        {
            return 1;
        }
        if (*tty_client(tty)).flags & CLIENT_UTF8 as uint64_t != 0 {
            return 0;
        }
        1
    }
}

/// The string to draw for the ACS key `ch`: the terminal's own translation
/// when it wants ACS, and the UTF-8 character otherwise. Nothing if neither
/// has one.
///
/// The terminal's own translation borrows from `tty`, so the answer lives only
/// as long as the caller keeps that terminal; the module's own table is
/// `'static`.
pub unsafe fn tty_acs_get<'a>(tty: *mut tty, ch: u_char) -> Option<&'a CStr> {
    unsafe {
        if tty_acs_needed(tty) != 0 {
            let acs = &tty_term_of(&*tty).acs[ch as usize];
            if acs[0] == 0 {
                return None;
            }
            return Some(CStr::from_ptr(acs.as_ptr()));
        }
        match tty_acs_table.binary_search_by_key(&ch, |entry| entry.key) {
            Ok(i) => Some(tty_acs_table[i].string),
            Err(_) => None,
        }
    }
}

/// The ACS key for the UTF-8 bytes in `s`, or -1 if there is none. Only two-
/// and three-byte strings have one.
pub fn tty_acs_reverse_get(s: &[u8]) -> c_int {
    let table: &[tty_acs_reverse_entry] = match s.len() {
        2 => &tty_acs_reverse2,
        3 => &tty_acs_reverse3,
        _ => return -1,
    };
    match table.binary_search_by(|entry| entry.string.to_bytes().cmp(s)) {
        Ok(i) => table[i].key as c_int,
        Err(_) => -1,
    }
}

#[cfg(test)]
#[path = "../tests/test_tty_acs.rs"]
mod tests;

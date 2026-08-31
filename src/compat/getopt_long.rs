use crate::ffi::{getenv, warnx};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};

/// One long option, as `getopt_long(3)` declares it. A table of them ends with
/// an entry whose name is null.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct option_t {
    pub name: *const c_char,
    pub has_arg: c_int,
    pub flag: *mut c_int,
    pub val: c_int,
}

pub const no_argument: c_int = 0;
pub const required_argument: c_int = 1;
pub const optional_argument: c_int = 2;

pub static mut BSDopterr: c_int = 1;
pub static mut BSDoptind: c_int = 1;
pub static mut BSDoptopt: c_int = '?' as c_int;
pub static mut BSDoptreset: c_int = 0;
pub static mut BSDoptarg: *mut c_char = null_mut::<c_char>();

/// Move every plain argument behind the options rather than stopping at the
/// first one.
pub const FLAG_PERMUTE: c_int = 0x1;
/// Hand every plain argument back as [`INORDER`] rather than stopping.
pub const FLAG_ALLARGS: c_int = 0x2;
/// One dash is enough in front of a long option.
pub const FLAG_LONGONLY: c_int = 0x4;

/// What an option nobody declared is answered as.
pub const BADCH: c_int = '?' as c_int;
/// What a plain argument is answered as under [`FLAG_ALLARGS`].
pub const INORDER: c_int = 1;

/// The empty string the place is given up to when there is nothing left of
/// the current argument.
pub const EMSG: *mut c_char = c"".as_ptr() as *mut c_char;

/// Where the parser has got to inside the argument it is reading. It points
/// into the caller's own argument list, which is what lets an option's
/// argument be handed back as a pointer into it.
static mut place: *mut c_char = EMSG;
/// The stretch of plain arguments waiting to be moved behind the options, or
/// -1 for each end that is not known yet.
static mut nonopt_start: c_int = -1;
static mut nonopt_end: c_int = -1;

const RECARGCHAR: &CStr = c"option requires an argument -- %c";
const RECARGSTRING: &CStr = c"option requires an argument -- %s";
const AMBIG: &CStr = c"ambiguous option -- %.*s";
const NOARG: &CStr = c"option doesn't take an argument -- %.*s";
const ILLOPTCHAR: &CStr = c"unknown option -- %c";
const ILLOPTSTRING: &CStr = c"unknown option -- %s";

/// The bytes of a C string, without its terminator.
unsafe fn bytes<'a>(p: *const c_char) -> &'a [u8] {
    unsafe { CStr::from_ptr(p).to_bytes() }
}

/// Where `ch` is in the option string, or nothing. `ch` is never the
/// terminator, which `strchr` would have answered the end of the string for.
unsafe fn find(options: *const c_char, ch: u8) -> Option<usize> {
    unsafe { bytes(options).iter().position(|&b| b == ch) }
}

/// The greatest common divisor of `a` and `b`.
fn gcd(mut a: c_int, mut b: c_int) -> c_int {
    let mut c = a % b;
    while c != 0 {
        a = b;
        b = c;
        c = a % b;
    }
    b
}

/// Swaps the block of plain arguments at `[start, end)` with the options that
/// follow it up to `opt_end`, rotating the whole stretch one cycle at a time
/// so that nothing needs a buffer.
fn permute_args(start: c_int, end: c_int, opt_end: c_int, nargv: &mut [*mut c_char]) {
    let nnonopts = end - start;
    let nopts = opt_end - end;
    let ncycle = gcd(nnonopts, nopts);
    let cyclelen = (opt_end - start) / ncycle;
    for i in 0..ncycle {
        let cstart = end + i;
        let mut pos = cstart;
        for _ in 0..cyclelen {
            if pos >= end {
                pos -= nnonopts;
            } else {
                pos += nopts;
            }
            nargv.swap(pos as usize, cstart as usize);
        }
    }
}

/// The long-option table, up to the entry that has no name.
unsafe fn table<'a>(long_options: *const option_t) -> &'a [option_t] {
    unsafe {
        let mut n = 0;
        while !(*long_options.add(n)).name.is_null() {
            n += 1;
        }
        ::core::slice::from_raw_parts(long_options, n)
    }
}

/// Reads the long option the place is at: the value it stands for, zero when
/// it wrote its value into the option's own flag, [`BADCH`] (or a colon, when
/// the option string asked for quiet reporting) when it is no option this
/// table knows, or -1 when `short_too` says it may be a short option after all.
///
/// A name may be shortened as long as it still fits only one option, and its
/// argument may be written after an equals sign or as the word that follows.
unsafe fn parse_long_options(
    nargv: &mut [*mut c_char],
    options: *const c_char,
    long_options: *const option_t,
    idx: *mut c_int,
    short_too: bool,
) -> c_int {
    unsafe {
        let quiet = *options as u8 == b':';
        let current = place;
        let whole = bytes(current);
        BSDoptind += 1;
        let (name, has_equal) = match whole.iter().position(|&b| b == b'=') {
            Some(at) => (&whole[..at], Some(current.add(at + 1))),
            None => (whole, None),
        };
        let entries = table(long_options);
        let mut found: Option<usize> = None;
        for (i, entry) in entries.iter().enumerate() {
            if !bytes(entry.name).starts_with(name) {
                continue;
            }
            if bytes(entry.name).len() == name.len() {
                found = Some(i);
                break;
            }
            if short_too && name.len() == 1 {
                continue;
            }
            if found.is_none() {
                found = Some(i);
                continue;
            }
            if BSDopterr != 0 && !quiet {
                warnx(AMBIG.as_ptr(), name.len() as c_int, current);
            }
            BSDoptopt = 0;
            return BADCH;
        }
        let Some(at) = found else {
            if short_too {
                BSDoptind -= 1;
                return -1;
            }
            if BSDopterr != 0 && !quiet {
                warnx(ILLOPTSTRING.as_ptr(), current);
            }
            BSDoptopt = 0;
            return BADCH;
        };
        let entry = &entries[at];
        let refused = |entry: &option_t| if entry.flag.is_null() { entry.val } else { 0 };
        if entry.has_arg == no_argument && has_equal.is_some() {
            if BSDopterr != 0 && !quiet {
                warnx(NOARG.as_ptr(), name.len() as c_int, current);
            }
            BSDoptopt = refused(entry);
            return if quiet { ':' as c_int } else { BADCH };
        }
        if entry.has_arg == required_argument || entry.has_arg == optional_argument {
            if let Some(value) = has_equal {
                BSDoptarg = value;
            } else if entry.has_arg == required_argument {
                BSDoptarg = nargv[BSDoptind as usize];
                BSDoptind += 1;
            }
        }
        if entry.has_arg == required_argument && BSDoptarg.is_null() {
            if BSDopterr != 0 && !quiet {
                warnx(RECARGSTRING.as_ptr(), current);
            }
            BSDoptopt = refused(entry);
            BSDoptind -= 1;
            return if quiet { ':' as c_int } else { BADCH };
        }
        if !idx.is_null() {
            *idx = at as c_int;
        }
        if !entry.flag.is_null() {
            *entry.flag = entry.val;
            return 0;
        }
        entry.val
    }
}

/// Reads the next option out of the argument list, answering -1 once there is
/// nothing left to read.
///
/// The argument list runs one past its arguments because that slot holds the
/// null terminator every `argv` carries, and a long option whose argument is
/// missing reads it to find out.
unsafe fn getopt_internal(
    nargv: &mut [*mut c_char],
    mut options: *const c_char,
    long_options: *const option_t,
    idx: *mut c_int,
    mut flags: c_int,
) -> c_int {
    unsafe {
        static mut posixly_correct: c_int = -1;
        if options.is_null() {
            return -1;
        }
        let nargc = nargv.len() as c_int - 1;
        if BSDoptind == 0 {
            BSDoptreset = 1;
            BSDoptind = BSDoptreset;
        }
        if posixly_correct == -1 || BSDoptreset != 0 {
            posixly_correct = !getenv(c"POSIXLY_CORRECT".as_ptr()).is_null() as c_int;
        }
        if *options as u8 == b'-' {
            flags |= FLAG_ALLARGS;
        } else if posixly_correct != 0 || *options as u8 == b'+' {
            flags &= !FLAG_PERMUTE;
        }
        if *options as u8 == b'+' || *options as u8 == b'-' {
            options = options.add(1);
        }
        let quiet = *options as u8 == b':';
        BSDoptarg = null_mut::<c_char>();
        if BSDoptreset != 0 {
            nonopt_end = -1;
            nonopt_start = -1;
        }
        while BSDoptreset != 0 || *place == 0 {
            BSDoptreset = 0;
            if BSDoptind >= nargc {
                place = EMSG;
                if nonopt_end != -1 {
                    permute_args(nonopt_start, nonopt_end, BSDoptind, nargv);
                    BSDoptind -= nonopt_end - nonopt_start;
                } else if nonopt_start != -1 {
                    BSDoptind = nonopt_start;
                }
                nonopt_end = -1;
                nonopt_start = -1;
                return -1;
            }
            place = nargv[BSDoptind as usize];
            let current = bytes(place);
            if current.first() != Some(&b'-')
                || (current.len() == 1 && find(options, b'-').is_none())
            {
                place = EMSG;
                if flags & FLAG_ALLARGS != 0 {
                    BSDoptarg = nargv[BSDoptind as usize];
                    BSDoptind += 1;
                    return INORDER;
                }
                if flags & FLAG_PERMUTE == 0 {
                    return -1;
                }
                if nonopt_start == -1 {
                    nonopt_start = BSDoptind;
                } else if nonopt_end != -1 {
                    permute_args(nonopt_start, nonopt_end, BSDoptind, nargv);
                    nonopt_start = BSDoptind - (nonopt_end - nonopt_start);
                    nonopt_end = -1;
                }
                BSDoptind += 1;
                continue;
            }
            if nonopt_start != -1 && nonopt_end == -1 {
                nonopt_end = BSDoptind;
            }
            if current.len() > 1 {
                place = place.add(1);
                if *place as u8 == b'-' && *place.add(1) == 0 {
                    BSDoptind += 1;
                    place = EMSG;
                    if nonopt_end != -1 {
                        permute_args(nonopt_start, nonopt_end, BSDoptind, nargv);
                        BSDoptind -= nonopt_end - nonopt_start;
                    }
                    nonopt_end = -1;
                    nonopt_start = -1;
                    return -1;
                }
            }
            break;
        }
        if !long_options.is_null()
            && place != nargv[BSDoptind as usize]
            && (*place as u8 == b'-' || flags & FLAG_LONGONLY != 0)
        {
            let mut short_too = false;
            if *place as u8 == b'-' {
                place = place.add(1);
            } else if *place as u8 != b':' && find(options, *place as u8).is_some() {
                short_too = true;
            }
            let optchar = parse_long_options(nargv, options, long_options, idx, short_too);
            if optchar != -1 {
                place = EMSG;
                return optchar;
            }
        }
        let optchar = *place as c_int;
        place = place.add(1);
        let known = if optchar == ':' as c_int || (optchar == '-' as c_int && *place != 0) {
            None
        } else {
            find(options, optchar as u8)
        };
        let Some(oli) = known else {
            if optchar == '-' as c_int && *place == 0 {
                return -1;
            }
            if *place == 0 {
                BSDoptind += 1;
            }
            if BSDopterr != 0 && !quiet {
                warnx(ILLOPTCHAR.as_ptr(), optchar);
            }
            BSDoptopt = optchar;
            return BADCH;
        };
        let opts = bytes(options);
        if !long_options.is_null() && optchar == 'W' as c_int && opts.get(oli + 1) == Some(&b';') {
            if *place == 0 {
                BSDoptind += 1;
                if BSDoptind >= nargc {
                    place = EMSG;
                    if BSDopterr != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt = optchar;
                    return if quiet { ':' as c_int } else { BADCH };
                }
                place = nargv[BSDoptind as usize];
            }
            let optchar = parse_long_options(nargv, options, long_options, idx, false);
            place = EMSG;
            return optchar;
        }
        if opts.get(oli + 1) != Some(&b':') {
            if *place == 0 {
                BSDoptind += 1;
            }
        } else {
            BSDoptarg = null_mut::<c_char>();
            if *place != 0 {
                BSDoptarg = place;
            } else if opts.get(oli + 2) != Some(&b':') {
                BSDoptind += 1;
                if BSDoptind >= nargc {
                    place = EMSG;
                    if BSDopterr != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt = optchar;
                    return if quiet { ':' as c_int } else { BADCH };
                }
                BSDoptarg = nargv[BSDoptind as usize];
            }
            place = EMSG;
            BSDoptind += 1;
        }
        optchar
    }
}

/// Reads the next short option out of the argument list.
pub unsafe fn BSDgetopt(nargv: &mut [*mut c_char], options: *const c_char) -> c_int {
    unsafe { getopt_internal(nargv, options, null::<option_t>(), null_mut::<c_int>(), 0) }
}

#[cfg(test)]
#[path = "../tests/test_compat_getopt_long.rs"]
mod tests;

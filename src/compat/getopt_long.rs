use crate::environ::process_environment_value;
use crate::ffi::warnx;
use ::core::ffi::{CStr, c_char, c_int};
use ::std::ffi::CString;

/// One long option, retaining the optional flag for the descriptor's lifetime.
/// A descriptor with no name terminates a table early.
pub struct option_t<'a> {
    pub name: Option<&'static CStr>,
    pub has_arg: c_int,
    pub flag: Option<&'a mut c_int>,
    pub val: c_int,
}

pub const no_argument: c_int = 0;
pub const required_argument: c_int = 1;
pub const optional_argument: c_int = 2;

pub static mut BSDopterr: c_int = 1;
pub static mut BSDoptind: c_int = 1;
pub static mut BSDoptopt: c_int = '?' as c_int;
pub static mut BSDoptreset: c_int = 0;
#[derive(Clone, Copy)]
struct ArgumentPosition {
    index: usize,
    offset: usize,
}

impl ArgumentPosition {
    fn read(self, arguments: &[CString]) -> &CStr {
        CStr::from_bytes_with_nul(&arguments[self.index].as_bytes_with_nul()[self.offset..])
            .expect("an argument suffix remains NUL-terminated")
    }

    fn advance(self, count: usize) -> Self {
        Self {
            offset: self.offset + count,
            ..self
        }
    }
}

static mut optarg: Option<ArgumentPosition> = None;

/// Borrows the last option's argument from the list being parsed.
pub unsafe fn BSDoptarg(arguments: &[CString]) -> Option<&CStr> {
    unsafe { optarg.map(|position| position.read(arguments)) }
}

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

/// The argument and byte offset being read, or no current argument.
static mut place: Option<ArgumentPosition> = None;

unsafe fn current_argument(arguments: &[CString]) -> &CStr {
    unsafe { place.map_or(c"", |position| position.read(arguments)) }
}

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

/// Where `ch` is in the option string, or nothing. `ch` is never the
/// terminator, which `strchr` would have answered the end of the string for.
fn find(options: &[u8], ch: u8) -> Option<usize> {
    options.iter().position(|&b| b == ch)
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
fn permute_args(start: c_int, end: c_int, opt_end: c_int, nargv: &mut [CString]) {
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

/// Reads the long option the place is at: the value it stands for, zero when
/// it wrote its value into the option's own flag, [`BADCH`] (or a colon, when
/// the option string asked for quiet reporting) when it is no option this
/// table knows, or -1 when `short_too` says it may be a short option after all.
///
/// A name may be shortened as long as it still fits only one option, and its
/// argument may be written after an equals sign or as the word that follows.
unsafe fn parse_long_options(
    nargv: &mut [CString],
    options: &[u8],
    long_options: &mut [option_t<'_>],
    idx: Option<&mut c_int>,
    short_too: bool,
) -> c_int {
    unsafe {
        let quiet = options.first() == Some(&b':');
        let position = place.expect("a long option has a current argument");
        let current = position.read(nargv);
        let whole = current.to_bytes();
        BSDoptind += 1;
        let (name, has_equal) = match whole.iter().position(|&b| b == b'=') {
            Some(at) => (&whole[..at], Some(position.advance(at + 1))),
            None => (whole, None),
        };
        let length = long_options
            .iter()
            .position(|entry| entry.name.is_none())
            .unwrap_or(long_options.len());
        let entries = &mut long_options[..length];
        let mut found: Option<usize> = None;
        for (i, entry) in entries.iter().enumerate() {
            if !entry.name.unwrap().to_bytes().starts_with(name) {
                continue;
            }
            if entry.name.unwrap().to_bytes().len() == name.len() {
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
                warnx(AMBIG.as_ptr(), name.len() as c_int, current.as_ptr());
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
                warnx(ILLOPTSTRING.as_ptr(), current.as_ptr());
            }
            BSDoptopt = 0;
            return BADCH;
        };
        let entry = &mut entries[at];
        let refused = |entry: &option_t| if entry.flag.is_none() { entry.val } else { 0 };
        if entry.has_arg == no_argument && has_equal.is_some() {
            if BSDopterr != 0 && !quiet {
                warnx(NOARG.as_ptr(), name.len() as c_int, current.as_ptr());
            }
            BSDoptopt = refused(entry);
            return if quiet { ':' as c_int } else { BADCH };
        }
        if entry.has_arg == required_argument || entry.has_arg == optional_argument {
            if let Some(value) = has_equal {
                optarg = Some(value);
            } else if entry.has_arg == required_argument {
                optarg = nargv.get(BSDoptind as usize).map(|_| ArgumentPosition {
                    index: BSDoptind as usize,
                    offset: 0,
                });
                BSDoptind += 1;
            }
        }
        if entry.has_arg == required_argument && optarg.is_none() {
            if BSDopterr != 0 && !quiet {
                warnx(RECARGSTRING.as_ptr(), current.as_ptr());
            }
            BSDoptopt = refused(entry);
            BSDoptind -= 1;
            return if quiet { ':' as c_int } else { BADCH };
        }
        if let Some(idx) = idx {
            *idx = at as c_int;
        }
        if let Some(flag) = entry.flag.as_deref_mut() {
            *flag = entry.val;
            return 0;
        }
        entry.val
    }
}

/// Reads the next option out of the argument list, answering -1 once there is
/// nothing left to read.
///
/// The argument strings remain owned while their order may change during
/// option permutation. A missing long-option argument is an absent entry.
unsafe fn getopt_internal(
    nargv: &mut [CString],
    options: Option<&CStr>,
    long_options: &mut [option_t<'_>],
    mut idx: Option<&mut c_int>,
    mut flags: c_int,
) -> c_int {
    unsafe {
        static mut posixly_correct: c_int = -1;
        let Some(options) = options else {
            return -1;
        };
        let mut options = options.to_bytes();
        let nargc = nargv.len() as c_int;
        if BSDoptind == 0 {
            BSDoptreset = 1;
            BSDoptind = BSDoptreset;
        }
        if posixly_correct == -1 || BSDoptreset != 0 {
            posixly_correct = process_environment_value(c"POSIXLY_CORRECT").is_some() as c_int;
        }
        if options.first() == Some(&b'-') {
            flags |= FLAG_ALLARGS;
        } else if posixly_correct != 0 || options.first() == Some(&b'+') {
            flags &= !FLAG_PERMUTE;
        }
        if options.first() == Some(&b'+') || options.first() == Some(&b'-') {
            options = &options[1..];
        }
        let quiet = options.first() == Some(&b':');
        optarg = None;
        if BSDoptreset != 0 {
            nonopt_end = -1;
            nonopt_start = -1;
        }
        while BSDoptreset != 0 || current_argument(nargv).is_empty() {
            BSDoptreset = 0;
            if BSDoptind >= nargc {
                place = None;
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
            place = Some(ArgumentPosition {
                index: BSDoptind as usize,
                offset: 0,
            });
            let current = current_argument(nargv).to_bytes();
            if current.first() != Some(&b'-')
                || (current.len() == 1 && find(options, b'-').is_none())
            {
                place = None;
                if flags & FLAG_ALLARGS != 0 {
                    optarg = nargv.get(BSDoptind as usize).map(|_| ArgumentPosition {
                        index: BSDoptind as usize,
                        offset: 0,
                    });
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
                place = place.map(|position| position.advance(1));
                if current_argument(nargv) == c"-" {
                    BSDoptind += 1;
                    place = None;
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
        if !long_options.is_empty()
            && place.is_some_and(|position| position.offset != 0)
            && (current_argument(nargv).to_bytes().first() == Some(&b'-')
                || flags & FLAG_LONGONLY != 0)
        {
            let mut short_too = false;
            if current_argument(nargv).to_bytes().first() == Some(&b'-') {
                place = place.map(|position| position.advance(1));
            } else if current_argument(nargv).to_bytes().first() != Some(&b':')
                && find(options, current_argument(nargv).to_bytes_with_nul()[0]).is_some()
            {
                short_too = true;
            }
            let optchar =
                parse_long_options(nargv, options, long_options, idx.as_deref_mut(), short_too);
            if optchar != -1 {
                place = None;
                return optchar;
            }
        }
        let optchar = current_argument(nargv).to_bytes_with_nul()[0] as c_char as c_int;
        place = place.map(|position| position.advance(1));
        let known = if optchar == ':' as c_int
            || (optchar == '-' as c_int && !current_argument(nargv).is_empty())
        {
            None
        } else {
            find(options, optchar as u8)
        };
        let Some(oli) = known else {
            if optchar == '-' as c_int && current_argument(nargv).is_empty() {
                return -1;
            }
            if current_argument(nargv).is_empty() {
                BSDoptind += 1;
            }
            if BSDopterr != 0 && !quiet {
                warnx(ILLOPTCHAR.as_ptr(), optchar);
            }
            BSDoptopt = optchar;
            return BADCH;
        };
        let opts = options;
        if !long_options.is_empty() && optchar == 'W' as c_int && opts.get(oli + 1) == Some(&b';') {
            if current_argument(nargv).is_empty() {
                BSDoptind += 1;
                if BSDoptind >= nargc {
                    place = None;
                    if BSDopterr != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt = optchar;
                    return if quiet { ':' as c_int } else { BADCH };
                }
                place = Some(ArgumentPosition {
                    index: BSDoptind as usize,
                    offset: 0,
                });
            }
            let optchar = parse_long_options(nargv, options, long_options, idx, false);
            place = None;
            return optchar;
        }
        if opts.get(oli + 1) != Some(&b':') {
            if current_argument(nargv).is_empty() {
                BSDoptind += 1;
            }
        } else {
            optarg = None;
            if !current_argument(nargv).is_empty() {
                optarg = place;
            } else if opts.get(oli + 2) != Some(&b':') {
                BSDoptind += 1;
                if BSDoptind >= nargc {
                    place = None;
                    if BSDopterr != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt = optchar;
                    return if quiet { ':' as c_int } else { BADCH };
                }
                optarg = nargv.get(BSDoptind as usize).map(|_| ArgumentPosition {
                    index: BSDoptind as usize,
                    offset: 0,
                });
            }
            place = None;
            BSDoptind += 1;
        }
        optchar
    }
}

/// Reads the next short option out of the argument list.
pub unsafe fn BSDgetopt(nargv: &mut [CString], options: &CStr) -> c_int {
    unsafe { getopt_internal(nargv, Some(options), &mut [], None, 0) }
}

#[cfg(test)]
#[path = "../tests/test_compat_getopt_long.rs"]
mod tests;

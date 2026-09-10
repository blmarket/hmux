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

pub const BSDopterr: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.bsdopterr);
pub const BSDoptind: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.bsdoptind);
pub const BSDoptopt: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.bsdoptopt);
pub const BSDoptreset: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.bsdoptreset);
#[derive(Clone, Copy)]
pub(crate) struct ArgumentPosition {
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

const optarg: crate::server_state::Value<Option<ArgumentPosition>> =
    crate::server_state::Value::new(|state| &state.optarg);

/// Borrows the last option's argument from the list being parsed.
pub unsafe fn BSDoptarg(arguments: &[CString]) -> Option<&CStr> {
    optarg.get().map(|position| position.read(arguments))
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

const place: crate::server_state::Value<Option<ArgumentPosition>> =
    crate::server_state::Value::new(|state| &state.place);

unsafe fn current_argument(arguments: &[CString]) -> &CStr {
    place.get().map_or(c"", |position| position.read(arguments))
}

const nonopt_start: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.nonopt_start);
const nonopt_end: crate::server_state::Value<c_int> =
    crate::server_state::Value::new(|state| &state.nonopt_end);

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
        let position = place.get().expect("a long option has a current argument");
        let current = position.read(nargv);
        let whole = current.to_bytes();
        {
            let value = 1;
            BSDoptind.with_mut(|current| *current += value)
        };
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
            if BSDopterr.get() != 0 && !quiet {
                warnx(AMBIG.as_ptr(), name.len() as c_int, current.as_ptr());
            }
            BSDoptopt.set(0);
            return BADCH;
        }
        let Some(at) = found else {
            if short_too {
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current -= value)
                };
                return -1;
            }
            if BSDopterr.get() != 0 && !quiet {
                warnx(ILLOPTSTRING.as_ptr(), current.as_ptr());
            }
            BSDoptopt.set(0);
            return BADCH;
        };
        let entry = &mut entries[at];
        let refused = |entry: &option_t| if entry.flag.is_none() { entry.val } else { 0 };
        if entry.has_arg == no_argument && has_equal.is_some() {
            if BSDopterr.get() != 0 && !quiet {
                warnx(NOARG.as_ptr(), name.len() as c_int, current.as_ptr());
            }
            BSDoptopt.set(refused(entry));
            return if quiet { ':' as c_int } else { BADCH };
        }
        if entry.has_arg == required_argument || entry.has_arg == optional_argument {
            if let Some(value) = has_equal {
                optarg.set(Some(value));
            } else if entry.has_arg == required_argument {
                optarg.set(
                    nargv
                        .get(BSDoptind.get() as usize)
                        .map(|_| ArgumentPosition {
                            index: BSDoptind.get() as usize,
                            offset: 0,
                        }),
                );
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
            }
        }
        if entry.has_arg == required_argument && optarg.get().is_none() {
            if BSDopterr.get() != 0 && !quiet {
                warnx(RECARGSTRING.as_ptr(), current.as_ptr());
            }
            BSDoptopt.set(refused(entry));
            {
                let value = 1;
                BSDoptind.with_mut(|current| *current -= value)
            };
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
        const getopt_posixly_correct: crate::server_state::Value<c_int> =
            crate::server_state::Value::new(|state| &state.getopt_posixly_correct);
        let Some(options) = options else {
            return -1;
        };
        let mut options = options.to_bytes();
        let nargc = nargv.len() as c_int;
        if BSDoptind.get() == 0 {
            BSDoptreset.set(1);
            BSDoptind.set(BSDoptreset.get());
        }
        if getopt_posixly_correct.get() == -1 || BSDoptreset.get() != 0 {
            getopt_posixly_correct
                .set(process_environment_value(c"POSIXLY_CORRECT").is_some() as c_int);
        }
        if options.first() == Some(&b'-') {
            flags |= FLAG_ALLARGS;
        } else if getopt_posixly_correct.get() != 0 || options.first() == Some(&b'+') {
            flags &= !FLAG_PERMUTE;
        }
        if options.first() == Some(&b'+') || options.first() == Some(&b'-') {
            options = &options[1..];
        }
        let quiet = options.first() == Some(&b':');
        optarg.set(None);
        if BSDoptreset.get() != 0 {
            nonopt_end.set(-1);
            nonopt_start.set(-1);
        }
        while BSDoptreset.get() != 0 || current_argument(nargv).is_empty() {
            BSDoptreset.set(0);
            if BSDoptind.get() >= nargc {
                place.set(None);
                if nonopt_end.get() != -1 {
                    permute_args(nonopt_start.get(), nonopt_end.get(), BSDoptind.get(), nargv);
                    {
                        let value = nonopt_end.get() - nonopt_start.get();
                        BSDoptind.with_mut(|current| *current -= value)
                    };
                } else if nonopt_start.get() != -1 {
                    BSDoptind.set(nonopt_start.get());
                }
                nonopt_end.set(-1);
                nonopt_start.set(-1);
                return -1;
            }
            place.set(Some(ArgumentPosition {
                index: BSDoptind.get() as usize,
                offset: 0,
            }));
            let current = current_argument(nargv).to_bytes();
            if current.first() != Some(&b'-')
                || (current.len() == 1 && find(options, b'-').is_none())
            {
                place.set(None);
                if flags & FLAG_ALLARGS != 0 {
                    optarg.set(
                        nargv
                            .get(BSDoptind.get() as usize)
                            .map(|_| ArgumentPosition {
                                index: BSDoptind.get() as usize,
                                offset: 0,
                            }),
                    );
                    {
                        let value = 1;
                        BSDoptind.with_mut(|current| *current += value)
                    };
                    return INORDER;
                }
                if flags & FLAG_PERMUTE == 0 {
                    return -1;
                }
                if nonopt_start.get() == -1 {
                    nonopt_start.set(BSDoptind.get());
                } else if nonopt_end.get() != -1 {
                    permute_args(nonopt_start.get(), nonopt_end.get(), BSDoptind.get(), nargv);
                    nonopt_start.set(BSDoptind.get() - (nonopt_end.get() - nonopt_start.get()));
                    nonopt_end.set(-1);
                }
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
                continue;
            }
            if nonopt_start.get() != -1 && nonopt_end.get() == -1 {
                nonopt_end.set(BSDoptind.get());
            }
            if current.len() > 1 {
                place.set(place.get().map(|position| position.advance(1)));
                if current_argument(nargv) == c"-" {
                    {
                        let value = 1;
                        BSDoptind.with_mut(|current| *current += value)
                    };
                    place.set(None);
                    if nonopt_end.get() != -1 {
                        permute_args(nonopt_start.get(), nonopt_end.get(), BSDoptind.get(), nargv);
                        {
                            let value = nonopt_end.get() - nonopt_start.get();
                            BSDoptind.with_mut(|current| *current -= value)
                        };
                    }
                    nonopt_end.set(-1);
                    nonopt_start.set(-1);
                    return -1;
                }
            }
            break;
        }
        if !long_options.is_empty()
            && place.get().is_some_and(|position| position.offset != 0)
            && (current_argument(nargv).to_bytes().first() == Some(&b'-')
                || flags & FLAG_LONGONLY != 0)
        {
            let mut short_too = false;
            if current_argument(nargv).to_bytes().first() == Some(&b'-') {
                place.set(place.get().map(|position| position.advance(1)));
            } else if current_argument(nargv).to_bytes().first() != Some(&b':')
                && find(options, current_argument(nargv).to_bytes_with_nul()[0]).is_some()
            {
                short_too = true;
            }
            let optchar =
                parse_long_options(nargv, options, long_options, idx.as_deref_mut(), short_too);
            if optchar != -1 {
                place.set(None);
                return optchar;
            }
        }
        let optchar = current_argument(nargv).to_bytes_with_nul()[0] as c_char as c_int;
        place.set(place.get().map(|position| position.advance(1)));
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
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
            }
            if BSDopterr.get() != 0 && !quiet {
                warnx(ILLOPTCHAR.as_ptr(), optchar);
            }
            BSDoptopt.set(optchar);
            return BADCH;
        };
        let opts = options;
        if !long_options.is_empty() && optchar == 'W' as c_int && opts.get(oli + 1) == Some(&b';') {
            if current_argument(nargv).is_empty() {
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
                if BSDoptind.get() >= nargc {
                    place.set(None);
                    if BSDopterr.get() != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt.set(optchar);
                    return if quiet { ':' as c_int } else { BADCH };
                }
                place.set(Some(ArgumentPosition {
                    index: BSDoptind.get() as usize,
                    offset: 0,
                }));
            }
            let optchar = parse_long_options(nargv, options, long_options, idx, false);
            place.set(None);
            return optchar;
        }
        if opts.get(oli + 1) != Some(&b':') {
            if current_argument(nargv).is_empty() {
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
            }
        } else {
            optarg.set(None);
            if !current_argument(nargv).is_empty() {
                optarg.set(place.get());
            } else if opts.get(oli + 2) != Some(&b':') {
                {
                    let value = 1;
                    BSDoptind.with_mut(|current| *current += value)
                };
                if BSDoptind.get() >= nargc {
                    place.set(None);
                    if BSDopterr.get() != 0 && !quiet {
                        warnx(RECARGCHAR.as_ptr(), optchar);
                    }
                    BSDoptopt.set(optchar);
                    return if quiet { ':' as c_int } else { BADCH };
                }
                optarg.set(
                    nargv
                        .get(BSDoptind.get() as usize)
                        .map(|_| ArgumentPosition {
                            index: BSDoptind.get() as usize,
                            offset: 0,
                        }),
                );
            }
            place.set(None);
            {
                let value = 1;
                BSDoptind.with_mut(|current| *current += value)
            };
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

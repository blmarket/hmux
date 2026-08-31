pub use crate::types::*;
pub const UNVIS_VALID: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const UNVIS_VALIDPUSH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const UNVIS_NOCHAR: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const UNVIS_SYNBAD: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const UNVIS_END: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const S_GROUND: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const S_START: ::core::ffi::c_int = 1;
pub const S_META: ::core::ffi::c_int = 2;
pub const S_META1: ::core::ffi::c_int = 3;
pub const S_CTRL: ::core::ffi::c_int = 4;
pub const S_OCTAL2: ::core::ffi::c_int = 5;
pub const S_OCTAL3: ::core::ffi::c_int = 6;
/// The decoder's position inside an escape sequence, mirroring the `S_*`
/// state codes the caller stores between calls.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ground,
    Start,
    Meta,
    Meta1,
    Ctrl,
    Octal2,
    Octal3,
}

impl State {
    fn from_code(code: ::core::ffi::c_int) -> Option<Self> {
        match code {
            S_GROUND => Some(State::Ground),
            S_START => Some(State::Start),
            S_META => Some(State::Meta),
            S_META1 => Some(State::Meta1),
            S_CTRL => Some(State::Ctrl),
            S_OCTAL2 => Some(State::Octal2),
            S_OCTAL3 => Some(State::Octal3),
            _ => None,
        }
    }

    fn code(self) -> ::core::ffi::c_int {
        match self {
            State::Ground => S_GROUND,
            State::Start => S_START,
            State::Meta => S_META,
            State::Meta1 => S_META1,
            State::Ctrl => S_CTRL,
            State::Octal2 => S_OCTAL2,
            State::Octal3 => S_OCTAL3,
        }
    }

    /// Whether the caller's character cell holds a partly built character that
    /// the next byte is folded into.
    fn holds_partial(self) -> bool {
        matches!(
            self,
            State::Meta1 | State::Ctrl | State::Octal2 | State::Octal3
        )
    }
}

/// What one decoding step did, mirroring the `UNVIS_*` return codes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Nothing to emit yet; feed the next byte.
    More,
    /// A character is ready.
    Valid,
    /// A character is ready and the byte just fed must be offered again.
    ValidPush,
    /// The byte encoded no character at all.
    NoChar,
    /// The sequence is not valid.
    SynBad,
}

impl Step {
    fn code(self) -> ::core::ffi::c_int {
        match self {
            Step::More => 0,
            Step::Valid => UNVIS_VALID,
            Step::ValidPush => UNVIS_VALIDPUSH,
            Step::NoChar => UNVIS_NOCHAR,
            Step::SynBad => UNVIS_SYNBAD,
        }
    }
}

/// Feed `c` to the decoder. `partial` is the character built so far (only
/// meaningful for the states where `holds_partial` is true); the returned byte
/// is the new value of the character cell, or `None` to leave it alone.
fn advance(state: Option<State>, c: u8, partial: u8) -> (State, Step, Option<u8>) {
    let Some(state) = state else {
        return (State::Ground, Step::SynBad, None);
    };
    match state {
        State::Ground => {
            if c == b'\\' {
                (State::Start, Step::More, Some(0))
            } else {
                (State::Ground, Step::Valid, Some(c))
            }
        }
        State::Start => match c {
            b'\\' => (State::Ground, Step::Valid, Some(c)),
            b'0'..=b'7' => (State::Octal2, Step::More, Some(c - b'0')),
            b'M' => (State::Meta, Step::More, Some(0o200)),
            b'^' => (State::Ctrl, Step::More, None),
            b'n' => (State::Ground, Step::Valid, Some(b'\n')),
            b'r' => (State::Ground, Step::Valid, Some(b'\r')),
            b'b' => (State::Ground, Step::Valid, Some(0x08)),
            b'a' => (State::Ground, Step::Valid, Some(0x07)),
            b'v' => (State::Ground, Step::Valid, Some(0x0b)),
            b't' => (State::Ground, Step::Valid, Some(b'\t')),
            b'f' => (State::Ground, Step::Valid, Some(0x0c)),
            b's' => (State::Ground, Step::Valid, Some(b' ')),
            b'E' => (State::Ground, Step::Valid, Some(0x1b)),
            b'\n' | b'$' => (State::Ground, Step::NoChar, None),
            _ => (State::Ground, Step::SynBad, None),
        },
        State::Meta => match c {
            b'-' => (State::Meta1, Step::More, None),
            b'^' => (State::Ctrl, Step::More, None),
            _ => (State::Ground, Step::SynBad, None),
        },
        State::Meta1 => (State::Ground, Step::Valid, Some(partial | c)),
        State::Ctrl => {
            let bits = if c == b'?' { 0o177 } else { c & 0o37 };
            (State::Ground, Step::Valid, Some(partial | bits))
        }
        State::Octal2 => match c {
            b'0'..=b'7' => (State::Octal3, Step::More, Some((partial << 3) | (c - b'0'))),
            _ => (State::Ground, Step::ValidPush, None),
        },
        State::Octal3 => match c {
            b'0'..=b'7' => (
                State::Ground,
                Step::Valid,
                Some((partial << 3) | (c - b'0')),
            ),
            _ => (State::Ground, Step::ValidPush, None),
        },
    }
}

/// Close the input. A half-built octal escape is still a character; any other
/// unfinished sequence is a syntax error and leaves the state untouched.
fn finish(state: Option<State>) -> (Option<State>, Step) {
    match state {
        Some(State::Octal2) | Some(State::Octal3) => (Some(State::Ground), Step::Valid),
        Some(State::Ground) => (state, Step::NoChar),
        _ => (state, Step::SynBad),
    }
}

/// Decode a whole visual-encoded string, returning the bytes it stands for, or
/// the bytes decoded before a syntax error.
fn decode(src: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut state = Some(State::Ground);
    let mut partial: u8 = 0;
    for &c in src {
        loop {
            let (next, step, value) = advance(state, c, partial);
            state = Some(next);
            if let Some(value) = value {
                partial = value;
            }
            match step {
                Step::Valid => {
                    out.push(partial);
                    break;
                }
                Step::ValidPush => out.push(partial),
                Step::More | Step::NoChar => break,
                Step::SynBad => return Err(out),
            }
        }
    }
    if finish(state).1 == Step::Valid {
        out.push(partial);
    }
    Ok(out)
}

/// Write as much of `out` as fits in the `sz` bytes at `dst`, in `strnunvis`'s
/// style: the last byte of the buffer is always a terminator, and one also
/// follows the copied bytes when they leave room for it.
unsafe fn store_bounded(dst: *mut ::core::ffi::c_char, sz: usize, out: &[u8]) {
    unsafe {
        if sz == 0 {
            return;
        }
        for (i, &b) in out.iter().take(sz - 1).enumerate() {
            *dst.add(i) = b as ::core::ffi::c_char;
        }
        if out.len() < sz {
            *dst.add(out.len()) = 0;
        }
        *dst.add(sz - 1) = 0;
    }
}

pub unsafe fn unvis(
    cp: *mut ::core::ffi::c_char,
    c: ::core::ffi::c_char,
    astate: *mut ::core::ffi::c_int,
    flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let state = State::from_code(*astate);
        if flag & UNVIS_END != 0 {
            let (next, step) = finish(state);
            if let Some(next) = next {
                *astate = next.code();
            }
            return step.code();
        }
        let partial = if state.is_some_and(State::holds_partial) {
            *cp as u8
        } else {
            0
        };
        let (next, step, value) = advance(state, c as u8, partial);
        *astate = next.code();
        if let Some(value) = value {
            *cp = value as ::core::ffi::c_char;
        }
        step.code()
    }
}

/// The string `src` stands for, or nothing when it holds a sequence that is
/// not valid. The C wrote the decoded bytes into a caller's buffer, terminated
/// them and answered their count; the string here stops at the first NUL the
/// escapes decode to, which is where reading that buffer back stopped.
pub fn strunvis(src: &::core::ffi::CStr) -> Option<::std::ffi::CString> {
    let out = decode(src.to_bytes()).ok()?;
    let end = out.iter().position(|&b| b == 0).unwrap_or(out.len());
    Some(::std::ffi::CString::new(&out[..end]).expect("the bytes stop at the first nul"))
}

pub unsafe fn strnunvis(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    sz: size_t,
) -> ssize_t {
    unsafe {
        match decode(::core::ffi::CStr::from_ptr(src).to_bytes()) {
            Ok(out) => {
                store_bounded(dst, sz, &out);
                out.len() as ssize_t
            }
            Err(partial) => {
                store_bounded(dst, sz, &partial);
                -1
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_unvis.rs"]
mod tests;

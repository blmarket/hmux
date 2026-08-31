use crate::ffi::{regcomp, regexec, regfree};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

/// How many match slots the substitution gives `regexec`: the whole match and
/// the nine groups a `\0` to `\9` in the replacement can name. The C tested
/// the group's number against this count before reading its slot; a single
/// digit is below ten whatever it is, so that test is gone.
const NMATCH: usize = 10;

/// A compiled pattern, freed when it goes out of scope.
struct Regex(regex_t);

impl Regex {
    /// Compiles `pattern` under `flags`, or nothing when it is not a pattern.
    fn compile(pattern: &CStr, flags: c_int) -> Option<Regex> {
        let mut r = regex_t::default();
        if unsafe { regcomp(&raw mut r, pattern.as_ptr(), flags) } != 0 {
            return None;
        }
        Some(Regex(r))
    }

    /// Where the pattern matches in `text` from the byte `at`, as offsets from
    /// `at` itself. The first slot is the whole match, the rest are the
    /// groups, and a group that matched nothing has both its offsets alike.
    fn exec(&self, text: &CStr, at: usize) -> Option<[regmatch_t; NMATCH]> {
        let mut m = [regmatch_t { rm_so: 0, rm_eo: 0 }; NMATCH];
        let matched = unsafe {
            regexec(
                &raw const self.0,
                text.as_ptr().add(at),
                NMATCH as size_t,
                m.as_mut_ptr(),
                0,
            ) == 0
        };
        matched.then_some(m)
    }
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe { regfree(&raw mut self.0) };
    }
}

/// Writes `with` onto the end of `out`, turning each `\0` to `\9` into what
/// the group of that number matched at `at`. A backslash in front of anything
/// else is dropped and the byte behind it written as it stands, so a digit
/// naming a group that matched nothing arrives without its backslash. A
/// backslash at the very end of `with` is kept.
fn expand(out: &mut Vec<u8>, with: &[u8], text: &[u8], at: usize, m: &[regmatch_t; NMATCH]) {
    let mut i = 0;
    while i < with.len() {
        let mut ch = with[i];
        if ch == b'\\' && i + 1 < with.len() {
            i += 1;
            ch = with[i];
            if ch.is_ascii_digit() {
                let group = &m[(ch - b'0') as usize];
                if group.rm_so != group.rm_eo {
                    let from = at + group.rm_so as usize;
                    let to = at + group.rm_eo as usize;
                    out.extend_from_slice(&text[from..to]);
                    i += 1;
                    continue;
                }
            }
        }
        out.push(ch);
        i += 1;
    }
}

/// What `text` becomes with every match of `pattern` replaced by `with`, or
/// nothing when `pattern` does not compile.
///
/// The walk keeps two positions: `start`, where the next match is looked for,
/// and `last`, where the text still to be copied begins. They part only over
/// an empty match, which steps `start` on a byte with `empty` set so that the
/// next look does not find the same nothing again; the byte stepped over is
/// copied by the run that follows. Every slice taken here is in range for that
/// reason — `last` is never past `start`, and a match's offsets are never past
/// the end of the text.
fn substitute(pattern: &CStr, with: &CStr, text: &CStr, flags: c_int) -> Option<Vec<u8>> {
    let bytes = text.to_bytes();
    if bytes.is_empty() {
        return Some(Vec::new());
    }
    let pattern_bytes = pattern.to_bytes();
    if pattern_bytes.is_empty() {
        return Some(bytes.to_vec());
    }
    let re = Regex::compile(pattern, flags)?;
    let anchored = pattern_bytes[0] == b'^';
    let end = bytes.len();
    let mut out = Vec::new();
    let mut start = 0;
    let mut last = 0;
    let mut empty = false;
    while start <= end {
        let Some(m) = re.exec(text, start) else {
            out.extend_from_slice(&bytes[start..end]);
            break;
        };
        let so = m[0].rm_so as usize;
        let eo = m[0].rm_eo as usize;
        out.extend_from_slice(&bytes[last..start + so]);
        let again = empty || start + so != last || so != eo;
        last = start + eo;
        if again {
            expand(&mut out, with.to_bytes(), bytes, start, &m);
            start += eo;
            empty = false;
        } else {
            start += eo + 1;
            empty = true;
        }
        if anchored {
            out.extend_from_slice(&bytes[start..end]);
            break;
        }
    }
    Some(out)
}

/// `text` with every match of `pattern` replaced by `with`, or nothing when
/// `pattern` does not compile.
pub fn regsub(pattern: &CStr, with: &CStr, text: &CStr, flags: c_int) -> Option<CString> {
    let answer = substitute(pattern, with, text, flags)?;
    Some(unsafe { CString::from_vec_unchecked(answer) })
}

#[cfg(test)]
#[path = "tests/test_regsub.rs"]
mod tests;

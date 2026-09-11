//! Owned matching with the platform POSIX regular-expression engine.

use crate::types::regmatch_t;
use core::ffi::{CStr, c_int};

/// An owned libc pattern, released when dropped.
pub struct CompiledRegex {
    buffer: libc::regex_t,
}

impl CompiledRegex {
    /// Compiles a NUL-terminated pattern using the platform POSIX regex engine.
    pub fn compile(pattern: &CStr, flags: c_int) -> Option<Self> {
        let mut buffer = unsafe { core::mem::zeroed::<libc::regex_t>() };
        if unsafe { libc::regcomp(&raw mut buffer, pattern.as_ptr(), flags) } != 0 {
            return None;
        }
        Some(Self { buffer })
    }

    /// Matches the suffix beginning at `at`, with offsets relative to that suffix.
    /// Only `REG_NOTBOL` and `REG_NOTEOL` execution flags are accepted. Invalid
    /// offsets or flags, and unsuccessful matches, return `None`.
    pub fn captures<const N: usize>(
        &self,
        text: &CStr,
        at: usize,
        flags: c_int,
    ) -> Option<[regmatch_t; N]> {
        if flags & !(libc::REG_NOTBOL | libc::REG_NOTEOL) != 0 {
            return None;
        }
        if at > text.to_bytes().len() {
            return None;
        }
        let suffix = &text[at..];
        let mut matches = [libc::regmatch_t { rm_so: 0, rm_eo: 0 }; N];
        let result = unsafe {
            libc::regexec(
                &raw const self.buffer,
                suffix.as_ptr(),
                N,
                matches.as_mut_ptr(),
                flags,
            )
        };
        (result == 0).then(|| matches.map(|m| regmatch_t { rm_so: m.rm_so, rm_eo: m.rm_eo }))
    }
}

impl Drop for CompiledRegex {
    fn drop(&mut self) {
        unsafe { libc::regfree(&raw mut self.buffer) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_patterns_match_bounded_suffixes_and_reject_invalid_requests() {
        let regex = CompiledRegex::compile(c"^(a+)(b)?$", libc::REG_EXTENDED).unwrap();
        let matches = regex.captures::<3>(c"xaa", 1, 0).unwrap();
        assert_eq!(
            matches.map(|m| (m.rm_so, m.rm_eo)),
            [(0, 2), (0, 2), (-1, -1)]
        );
        assert!(regex.captures::<3>(c"xaa", usize::MAX, 0).is_none());
        assert!(regex.captures::<3>(c"aa", 0, libc::REG_STARTEND).is_none());
        assert!(regex.captures::<3>(c"aa", 0, libc::REG_NOTBOL).is_none());
        assert!(CompiledRegex::compile(c"[", libc::REG_EXTENDED).is_none());
        let empty = CompiledRegex::compile(c"^$", libc::REG_EXTENDED).unwrap();
        assert!(empty.captures::<0>(c"abc", 3, 0).is_some());
        assert!(empty.captures::<0>(c"abc", 4, 0).is_none());
    }

    #[test]
    fn owned_compilation_reports_capture_offsets() {
        let regex = CompiledRegex::compile(c"(a)(b)", libc::REG_EXTENDED).unwrap();
        let matches = regex.captures::<3>(c"ab", 0, 0).unwrap();
        assert_eq!(
            matches.map(|m| (m.rm_so, m.rm_eo)),
            [(0, 2), (0, 1), (1, 2)]
        );
    }
}

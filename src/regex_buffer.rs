//! Stable inspection of the platform regex compilation buffer.

use crate::types::regmatch_t;
use core::ffi::{CStr, c_char, c_int, c_ulong, c_void};

#[derive(BitfieldStruct)]
#[repr(C)]
struct RawRegex {
    buffer: *mut c_void,
    allocated: c_ulong,
    used: c_ulong,
    syntax: c_ulong,
    fastmap: *mut core::ffi::c_char,
    translate: *mut core::ffi::c_uchar,
    re_nsub: usize,
    #[bitfield(name = "can_be_null", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "regs_allocated", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "fastmap_accurate", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "no_sub", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "not_bol", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "not_eol", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "newline_anchor", ty = "::core::ffi::c_uint", bits = "7..=7")]
    can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [u8; 1],
    #[bitfield(padding)]
    c2rust_padding: [u8; 7],
}
/// Zero-initialized state supplied to the platform compiler.
impl Default for RawRegex {
    fn default() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            allocated: 0,
            used: 0,
            syntax: 0,
            fastmap: core::ptr::null_mut(),
            translate: core::ptr::null_mut(),
            re_nsub: 0,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0;
                1],
            c2rust_padding: [0; 7],
        }
    }
}
unsafe extern "C" {
    fn regcomp(buffer: *mut RawRegex, pattern: *const c_char, flags: c_int) -> c_int;
    fn regexec(
        buffer: *const RawRegex,
        text: *const c_char,
        count: usize,
        matches: *mut regmatch_t,
        flags: c_int,
    ) -> c_int;
    fn regfree(buffer: *mut RawRegex);
}

/// An owned libc pattern, released when dropped.
pub struct CompiledRegex {
    buffer: RawRegex,
}

impl CompiledRegex {
    /// Compiles a NUL-terminated pattern using the platform POSIX regex engine.
    pub fn compile(pattern: &CStr, flags: c_int) -> Option<Self> {
        let mut buffer = RawRegex::default();
        if unsafe { regcomp(&raw mut buffer, pattern.as_ptr(), flags) } != 0 {
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
        let mut matches = [regmatch_t { rm_so: 0, rm_eo: 0 }; N];
        let result = unsafe {
            regexec(
                &raw const self.buffer,
                suffix.as_ptr(),
                N,
                matches.as_mut_ptr(),
                flags,
            )
        };
        (result == 0).then_some(matches)
    }
}

impl Drop for CompiledRegex {
    fn drop(&mut self) {
        unsafe { regfree(&raw mut self.buffer) };
    }
}

/// The storage and flags carried by a compiled POSIX regular expression.
pub trait RegexBuffer {
    /// Compiles an owned POSIX pattern, returning `None` for an invalid pattern.
    fn compile_regex(pattern: &CStr, flags: c_int) -> Option<Self>
    where
        Self: Sized;

    /// Matches a bounded suffix and returns owned offsets relative to that suffix.
    /// Accepts only `REG_NOTBOL` and `REG_NOTEOL` execution flags. Invalid
    /// offsets or flags, and unsuccessful matches, return `None`.
    fn regex_captures<const N: usize>(
        &self,
        text: &CStr,
        at: usize,
        flags: c_int,
    ) -> Option<[regmatch_t; N]>
    where
        Self: Sized;

    /// Returns the allocated automaton capacity.
    fn regex_buffer_allocated(&self) -> u64;

    /// Returns the used automaton capacity.
    fn regex_buffer_used(&self) -> u64;

    /// Returns the compilation syntax flags.
    fn regex_buffer_syntax(&self) -> u64;

    /// Returns the number of subexpressions.
    fn regex_buffer_subexpressions(&self) -> usize;

    /// Returns whether the expression may match an empty string.
    fn regex_buffer_can_be_null(&self) -> u8;

    /// Returns the register allocation mode.
    fn regex_buffer_registers_allocated(&self) -> u8;

    /// Returns whether the fast lookup map is current.
    fn regex_buffer_fastmap_accurate(&self) -> u8;

    /// Returns whether subexpression reporting is disabled.
    fn regex_buffer_no_subexpressions(&self) -> u8;

    /// Returns whether matching is not at the beginning of a line.
    fn regex_buffer_not_beginning_of_line(&self) -> u8;

    /// Returns whether matching is not at the end of a line.
    fn regex_buffer_not_end_of_line(&self) -> u8;

    /// Returns whether newlines act as anchors.
    fn regex_buffer_newline_anchor(&self) -> u8;
}

impl RegexBuffer for CompiledRegex {
    fn compile_regex(pattern: &CStr, flags: c_int) -> Option<Self> {
        Self::compile(pattern, flags)
    }
    fn regex_captures<const N: usize>(
        &self,
        text: &CStr,
        at: usize,
        flags: c_int,
    ) -> Option<[regmatch_t; N]> {
        self.captures(text, at, flags)
    }
    fn regex_buffer_allocated(&self) -> u64 {
        self.buffer.allocated
    }
    fn regex_buffer_used(&self) -> u64 {
        self.buffer.used
    }
    fn regex_buffer_syntax(&self) -> u64 {
        self.buffer.syntax
    }
    fn regex_buffer_subexpressions(&self) -> usize {
        self.buffer.re_nsub
    }
    fn regex_buffer_can_be_null(&self) -> u8 {
        self.buffer.can_be_null() as u8
    }
    fn regex_buffer_registers_allocated(&self) -> u8 {
        self.buffer.regs_allocated() as u8
    }
    fn regex_buffer_fastmap_accurate(&self) -> u8 {
        self.buffer.fastmap_accurate() as u8
    }
    fn regex_buffer_no_subexpressions(&self) -> u8 {
        self.buffer.no_sub() as u8
    }
    fn regex_buffer_not_beginning_of_line(&self) -> u8 {
        self.buffer.not_bol() as u8
    }
    fn regex_buffer_not_end_of_line(&self) -> u8 {
        self.buffer.not_eol() as u8
    }
    fn regex_buffer_newline_anchor(&self) -> u8 {
        self.buffer.newline_anchor() as u8
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
    fn owned_compilation_exposes_metadata_and_match_offsets() {
        let regex = CompiledRegex::compile_regex(c"(a)(b)", libc::REG_EXTENDED).unwrap();
        assert_eq!(regex.regex_buffer_subexpressions(), 2);
        assert!(regex.regex_buffer_used() <= regex.regex_buffer_allocated());
        let matches = regex.regex_captures::<3>(c"ab", 0, 0).unwrap();
        assert_eq!(
            matches.map(|m| (m.rm_so, m.rm_eo)),
            [(0, 2), (0, 1), (1, 2)]
        );
    }
}

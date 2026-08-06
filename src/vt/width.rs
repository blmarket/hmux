//! The width policy: how many cells a character or grapheme cluster occupies.
//!
//! Width is its own concern, not part of the screen trait. tmux exposes it as
//! options (`codepoint-widths`, `variation-selector-always-wide`), so the
//! policy has to be addressable on its own rather than buried in whichever
//! emulator owns the grid.
//!
//! Today the tables are libghostty-vt's, reached through the same bindings the
//! screen backend uses. That is why those two options are currently inert, and
//! it is the reason the seam names the policy separately: replacing the tables
//! should not mean replacing the screen.

use ghostty_sys::ffi;

/// The terminal-cell width of one Unicode codepoint.
///
/// The result is always 0, 1, or 2. This function is total: values above
/// U+10FFFF have width one.
#[must_use]
pub(crate) fn codepoint_width(codepoint: u32) -> u8 {
    // SAFETY: the function accepts every `u32`, has no pointer arguments, and
    // is documented by libghostty-vt as pure and thread-safe.
    unsafe { ffi::ghostty_unicode_codepoint_width(codepoint) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_width_uses_the_engines_terminal_table() {
        assert_eq!(codepoint_width('a' as u32), 1);
        assert_eq!(codepoint_width('界' as u32), 2);
        assert_eq!(codepoint_width(0x0301), 0); // combining acute accent
        assert_eq!(codepoint_width('ㄱ' as u32), 2);
        assert_eq!(codepoint_width(0x1161), 0); // conjoining Hangul Jamo vowel
        assert_eq!(codepoint_width(0x110000), 1); // total for invalid codepoints
    }
}

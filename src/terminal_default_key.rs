//! Stable access to terminal default-key table entries.

use crate::text::key_code;
use crate::types::tty_code_code;
use core::ffi::CStr;

/// A terminfo capability mapped to a key code.
pub trait TerminalDefaultCodeKey {
    /// Builds a terminfo key mapping.
    fn from_terminal_default_code_key(code: tty_code_code, key: key_code) -> Self
    where
        Self: Sized;

    /// Returns the terminfo capability code.
    fn terminal_default_key_code(&self) -> tty_code_code;

    /// Returns the key produced by the capability.
    fn terminal_default_code_key(&self) -> key_code;
}

/// A raw terminal byte sequence mapped to a key code.
pub trait TerminalDefaultRawKey {
    /// Builds a raw terminal key mapping.
    fn from_terminal_default_raw_key(string: &'static CStr, key: key_code) -> Self
    where
        Self: Sized;

    /// Returns the terminal byte sequence.
    fn terminal_default_raw_sequence(&self) -> &CStr;

    /// Returns the key produced by the sequence.
    fn terminal_default_raw_key(&self) -> key_code;
}

/// An xterm modifier template mapped to a key code.
pub trait TerminalDefaultXtermKey {
    /// Builds an xterm key mapping.
    fn from_terminal_default_xterm_key(template: &'static CStr, key: key_code) -> Self
    where
        Self: Sized;

    /// Returns the sequence template containing the modifier placeholder.
    fn terminal_default_xterm_template(&self) -> &CStr;

    /// Returns the key produced by the sequence.
    fn terminal_default_xterm_key(&self) -> key_code;
}

impl TerminalDefaultCodeKey for crate::types::tty_default_key_code {
    fn from_terminal_default_code_key(code: tty_code_code, key: key_code) -> Self {
        Self { code, key }
    }
    fn terminal_default_key_code(&self) -> tty_code_code {
        self.code
    }
    fn terminal_default_code_key(&self) -> key_code {
        self.key
    }
}

impl TerminalDefaultRawKey for crate::types::tty_default_key_raw {
    fn from_terminal_default_raw_key(string: &'static CStr, key: key_code) -> Self {
        Self { string, key }
    }
    fn terminal_default_raw_sequence(&self) -> &CStr {
        self.string
    }
    fn terminal_default_raw_key(&self) -> key_code {
        self.key
    }
}

impl TerminalDefaultXtermKey for crate::types::tty_default_key_xterm {
    fn from_terminal_default_xterm_key(template: &'static CStr, key: key_code) -> Self {
        Self { template, key }
    }
    fn terminal_default_xterm_template(&self) -> &CStr {
        self.template
    }
    fn terminal_default_xterm_key(&self) -> key_code {
        self.key
    }
}

impl crate::types::tty_default_key_code {
    /// Builds a static terminfo capability mapping.
    pub const fn new(code: tty_code_code, key: key_code) -> Self {
        Self { code, key }
    }
}

impl crate::types::tty_default_key_raw {
    /// Builds a static raw terminal sequence mapping.
    pub const fn new(string: &'static CStr, key: key_code) -> Self {
        Self { string, key }
    }
}

impl crate::types::tty_default_key_xterm {
    /// Builds a static xterm modifier-template mapping.
    pub const fn new(template: &'static CStr, key: key_code) -> Self {
        Self { template, key }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{tty_default_key_code, tty_default_key_raw, tty_default_key_xterm};

    #[test]
    fn const_entries_expose_their_mappings() {
        const CODE: tty_default_key_code = tty_default_key_code::new(11, 101);
        const RAW: tty_default_key_raw = tty_default_key_raw::new(c"\x1b[A", 102);
        const XTERM: tty_default_key_xterm = tty_default_key_xterm::new(c"\x1b[1;_A", 103);
        assert_eq!(CODE.terminal_default_key_code(), 11);
        assert_eq!(CODE.terminal_default_code_key(), 101);
        assert_eq!(RAW.terminal_default_raw_sequence(), c"\x1b[A");
        assert_eq!(RAW.terminal_default_raw_key(), 102);
        assert_eq!(XTERM.terminal_default_xterm_template(), c"\x1b[1;_A");
        assert_eq!(XTERM.terminal_default_xterm_key(), 103);
    }
}

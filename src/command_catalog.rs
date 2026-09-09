//! Lookup and enumeration of registered commands.

use crate::cmd::cmd_table;
use crate::{CommandEntry, RustCommandEntry};
use core::ffi::CStr;
use std::ffi::CString;

/// The catalog shared by command parsing, completion, and introspection.
pub trait CommandCatalog {
    /// The command-entry implementation held by this catalog.
    type Entry: CommandEntry + 'static;

    /// Returns every registered entry in catalog order.
    fn entries(&self) -> &'static [&'static Self::Entry];

    /// Resolves an exact alias, exact name, or unambiguous name prefix.
    fn find(&self, name: &CStr) -> Result<&'static Self::Entry, CString>;
}

/// The command catalog registered by the Rust hmux engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustCommandCatalog;

impl CommandCatalog for RustCommandCatalog {
    type Entry = RustCommandEntry;

    fn entries(&self) -> &'static [&'static Self::Entry] {
        cmd_table
    }

    fn find(&self, name: &CStr) -> Result<&'static Self::Entry, CString> {
        find_entry(self.entries(), name)
    }
}

fn find_entry<E: CommandEntry + 'static>(
    entries: &'static [&'static E],
    name: &CStr,
) -> Result<&'static E, CString> {
    let wanted = name.to_bytes();
    let mut found = None;
    let mut ambiguous = false;
    for &entry in entries {
        if entry.alias().map(CStr::to_bytes) == Some(wanted) {
            return Ok(entry);
        }
        let this = entry.name().to_bytes();
        if this.starts_with(wanted) {
            ambiguous |= found.is_some();
            found = Some(entry);
            if this == wanted {
                return Ok(entry);
            }
        }
    }
    if ambiguous {
        let mut message = b"ambiguous command: ".to_vec();
        message.extend_from_slice(wanted);
        message.extend_from_slice(b", could be: ");
        let mut separator = b"".as_slice();
        for entry in entries {
            let candidate = entry.name().to_bytes();
            if candidate.starts_with(wanted) {
                message.extend_from_slice(separator);
                message.extend_from_slice(candidate);
                separator = b", ";
            }
        }
        return Err(CString::new(message).expect("command names contain no NUL"));
    }
    found.ok_or_else(|| {
        let mut message = b"unknown command: ".to_vec();
        message.extend_from_slice(wanted);
        CString::new(message).expect("command names contain no NUL")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_names_aliases_and_prefixes() {
        let catalog = RustCommandCatalog;
        assert_eq!(
            catalog.find(c"list-buffers").unwrap().name(),
            c"list-buffers"
        );
        assert_eq!(catalog.find(c"lsb").unwrap().name(), c"list-buffers");
        assert_eq!(catalog.find(c"list-bu").unwrap().name(), c"list-buffers");
    }

    #[test]
    fn reports_unknown_and_ambiguous_names() {
        let catalog = RustCommandCatalog;
        assert_eq!(
            catalog.find(c"no-such-command").err().unwrap(),
            c"unknown command: no-such-command"
        );
        assert!(
            catalog
                .find(c"list-")
                .err()
                .unwrap()
                .to_bytes()
                .starts_with(b"ambiguous command: list-, could be: ")
        );
    }
}

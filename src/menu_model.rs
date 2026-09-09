//! Stable access to built menus and their owned entries.

use crate::text::key_code;
use crate::types::u_int;
use core::ffi::CStr;
use std::ffi::CString;

/// One expanded, owned entry in a menu.
pub trait MenuEntry {
    /// Builds an entry from its displayed name, key, and command.
    fn from_menu_entry(name: Option<CString>, key: key_code, command: Option<CString>) -> Self
    where
        Self: Sized;

    /// Returns the displayed name, or nothing for a separator.
    fn menu_entry_name(&self) -> Option<&CStr>;

    /// Returns the key which selects this entry.
    fn menu_entry_key(&self) -> key_code;

    /// Returns the command run by this entry, when it has one.
    fn menu_entry_command(&self) -> Option<&CStr>;
}

/// An owned menu, including its title, measured width, and entries.
pub trait Menu {
    /// The owned entry type stored by this menu implementation.
    type Entry: MenuEntry;

    /// Builds an empty menu with its already measured title width.
    fn from_menu(title: CString, width: u_int) -> Self
    where
        Self: Sized;

    /// Returns the title drawn in the menu border.
    fn menu_title(&self) -> &CStr;

    /// Returns the widest formatted line in the menu.
    fn menu_width(&self) -> u_int;

    /// Updates the widest formatted line in the menu.
    fn set_menu_width(&mut self, width: u_int);

    /// Returns the number of entries in the menu.
    fn menu_entry_count(&self) -> usize;

    /// Returns the displayed name at `index`, or nothing for a separator.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the menu.
    fn menu_entry_name(&self, index: usize) -> Option<&CStr>;

    /// Returns the selection key at `index`.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the menu.
    fn menu_entry_key(&self, index: usize) -> key_code;

    /// Returns the command at `index`, when the entry has one.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the menu.
    fn menu_entry_command(&self, index: usize) -> Option<&CStr>;

    /// Appends an owned entry to the menu.
    fn push_menu_entry(&mut self, entry: Self::Entry);

    /// Returns whether the menu has no entries.
    fn menu_is_empty(&self) -> bool {
        self.menu_entry_count() == 0
    }
}

impl MenuEntry for crate::types::menu_entry {
    fn from_menu_entry(name: Option<CString>, key: key_code, command: Option<CString>) -> Self {
        Self { name, key, command }
    }

    fn menu_entry_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn menu_entry_key(&self) -> key_code {
        self.key
    }

    fn menu_entry_command(&self) -> Option<&CStr> {
        self.command.as_deref()
    }
}

impl Menu for crate::types::menu {
    type Entry = crate::types::menu_entry;

    fn from_menu(title: CString, width: u_int) -> Self {
        Self {
            title: Some(title),
            items: Vec::new(),
            width,
        }
    }

    fn menu_title(&self) -> &CStr {
        self.title.as_deref().expect("a built menu carries a title")
    }

    fn menu_width(&self) -> u_int {
        self.width
    }

    fn set_menu_width(&mut self, width: u_int) {
        self.width = width;
    }

    fn menu_entry_count(&self) -> usize {
        self.items.len()
    }

    fn menu_entry_name(&self, index: usize) -> Option<&CStr> {
        self.items[index].name.as_deref()
    }

    fn menu_entry_key(&self, index: usize) -> key_code {
        self.items[index].key
    }

    fn menu_entry_command(&self, index: usize) -> Option<&CStr> {
        self.items[index].command.as_deref()
    }

    fn push_menu_entry(&mut self, entry: Self::Entry) {
        self.items.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{menu, menu_entry};

    #[test]
    fn menu_owns_and_exposes_its_entries() {
        let mut menu = menu::from_menu(c"title".to_owned(), 5);
        menu.push_menu_entry(menu_entry::from_menu_entry(
            Some(c"run".to_owned()),
            b'r' as key_code,
            Some(c"display-message run".to_owned()),
        ));
        menu.push_menu_entry(menu_entry::default());
        menu.set_menu_width(9);

        assert_eq!(menu.menu_title(), c"title");
        assert_eq!(menu.menu_width(), 9);
        assert_eq!(menu.menu_entry_count(), 2);
        assert_eq!(menu.menu_entry_name(0), Some(c"run"));
        assert_eq!(menu.menu_entry_key(0), b'r' as key_code);
        assert_eq!(
            menu.menu_entry_command(0),
            Some(c"display-message run" as &CStr)
        );
        assert!(menu.menu_entry_name(1).is_none());
    }
}

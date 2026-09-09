//! Stable access to borrowed menu item templates.

use core::ffi::CStr;

/// A borrowed menu item name, key, and optional command.
pub trait MenuItem<'a> {
    /// Builds a menu item template.
    fn from_menu_item(name: Option<&'a CStr>, key: u64, command: Option<&'a CStr>) -> Self
    where
        Self: Sized;

    /// Returns the displayed name template.
    fn menu_item_name(&self) -> Option<&'a CStr>;

    /// Replaces the displayed name template.
    fn set_menu_item_name(&mut self, name: Option<&'a CStr>);

    /// Returns the shortcut key.
    fn menu_item_key(&self) -> u64;

    /// Replaces the shortcut key.
    fn set_menu_item_key(&mut self, key: u64);

    /// Returns the optional command template.
    fn menu_item_command(&self) -> Option<&'a CStr>;

    /// Replaces the optional command template.
    fn set_menu_item_command(&mut self, command: Option<&'a CStr>);
}

impl<'a> MenuItem<'a> for crate::types::menu_item<'a> {
    fn from_menu_item(name: Option<&'a CStr>, key: u64, command: Option<&'a CStr>) -> Self {
        Self { name, key, command }
    }

    fn menu_item_name(&self) -> Option<&'a CStr> {
        self.name
    }
    fn set_menu_item_name(&mut self, name: Option<&'a CStr>) {
        self.name = name;
    }
    fn menu_item_key(&self) -> u64 {
        self.key
    }
    fn set_menu_item_key(&mut self, key: u64) {
        self.key = key;
    }
    fn menu_item_command(&self) -> Option<&'a CStr> {
        self.command
    }
    fn set_menu_item_command(&mut self, command: Option<&'a CStr>) {
        self.command = command;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::menu_item;

    #[test]
    fn borrowed_fields_round_trip_and_update() {
        let mut item = menu_item::from_menu_item(Some(c"Open"), b'o' as u64, Some(c"open"));
        assert_eq!(item.menu_item_name(), Some(c"Open"));
        assert_eq!(item.menu_item_key(), b'o' as u64);
        assert_eq!(item.menu_item_command(), Some(c"open"));
        item.set_menu_item_name(None);
        item.set_menu_item_key(0);
        item.set_menu_item_command(None);
        assert_eq!(item.menu_item_name(), None);
        assert_eq!(item.menu_item_key(), 0);
        assert_eq!(item.menu_item_command(), None);
    }
}

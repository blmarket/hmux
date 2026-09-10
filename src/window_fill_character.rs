//! Optional glyph used outside a window's pane layout.

use crate::text::utf8_data;

/// Storage for a window's optional fill character.
pub trait WindowFillCharacterState: Default {
    /// Returns the fill character, if one is configured.
    fn fill_character(&self) -> Option<utf8_data>;

    /// Replaces or clears the fill character.
    fn set_fill_character(&mut self, character: Option<utf8_data>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_can_be_replaced_and_cleared() {
        let mut state = crate::types::window::default();
        assert!(state.fill_character().is_none());
        let mut character = utf8_data::default();
        character.data[0] = b'#';
        character.have = 1;
        character.size = 1;
        character.width = 1;
        state.set_fill_character(Some(character));
        assert_eq!(state.fill_character().unwrap().data[0], b'#');
        state.set_fill_character(None);
        assert!(state.fill_character().is_none());
    }
}

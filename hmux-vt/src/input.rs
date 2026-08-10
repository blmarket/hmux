//! Key and mouse identities, and the bytes they turn into.
//!
//! Encoding *reads* screen state — the cursor-key mode, the keypad mode, which
//! mouse protocol the program asked for — so the mouse encoder is a method on
//! [`crate::screen::PaneScreen`], which tracks those modes. This module holds the
//! values that encoder takes and the one keyless encoding that needs no pane.

/// A physical key identity, independent of the layout that produced it.
///
/// The code is opaque: only the constants below and [`Key::from_ascii`], which
/// maps a US-layout character onto the key that bears it, name a key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Key(i32);

impl Key {
    pub(crate) const UNIDENTIFIED: Key = Key(0);
    pub(crate) const BACKQUOTE: Key = Key(1);
    pub(crate) const BACKSLASH: Key = Key(2);
    pub(crate) const BRACKET_LEFT: Key = Key(3);
    pub(crate) const BRACKET_RIGHT: Key = Key(4);
    pub(crate) const COMMA: Key = Key(5);
    pub(crate) const DIGIT_0: Key = Key(6);
    pub(crate) const EQUAL: Key = Key(16);
    pub(crate) const A: Key = Key(20);
    pub(crate) const MINUS: Key = Key(46);
    pub(crate) const PERIOD: Key = Key(47);
    pub(crate) const QUOTE: Key = Key(48);
    pub(crate) const SEMICOLON: Key = Key(49);
    pub(crate) const SLASH: Key = Key(50);
    pub const BACKSPACE: Key = Key(53);
    pub const ENTER: Key = Key(58);
    pub(crate) const SPACE: Key = Key(63);
    pub const TAB: Key = Key(64);
    pub const DELETE: Key = Key(68);
    pub const END: Key = Key(69);
    pub const HOME: Key = Key(71);
    pub const INSERT: Key = Key(72);
    pub const PAGE_DOWN: Key = Key(73);
    pub const PAGE_UP: Key = Key(74);
    pub const ARROW_DOWN: Key = Key(75);
    pub const ARROW_LEFT: Key = Key(76);
    pub const ARROW_RIGHT: Key = Key(77);
    pub const ARROW_UP: Key = Key(78);
    pub(crate) const NUMPAD_0: Key = Key(80);
    pub const NUMPAD_ADD: Key = Key(90);
    pub const NUMPAD_DECIMAL: Key = Key(95);
    pub const NUMPAD_DIVIDE: Key = Key(96);
    pub const NUMPAD_ENTER: Key = Key(97);
    pub const NUMPAD_MULTIPLY: Key = Key(104);
    pub const NUMPAD_SUBTRACT: Key = Key(107);
    pub const ESCAPE: Key = Key(120);
    pub(crate) const F1: Key = Key(121);

    /// Map a printable US-layout ASCII character to the key that bears it.
    pub fn from_ascii(ch: char) -> Key {
        match ch {
            '`' | '~' => Key::BACKQUOTE,
            '\\' | '|' => Key::BACKSLASH,
            '[' | '{' => Key::BRACKET_LEFT,
            ']' | '}' => Key::BRACKET_RIGHT,
            ',' | '<' => Key::COMMA,
            '0'..='9' => Key(Key::DIGIT_0.0 + (ch as i32 - '0' as i32)),
            '=' | '+' => Key::EQUAL,
            'a'..='z' => Key(Key::A.0 + (ch as i32 - 'a' as i32)),
            'A'..='Z' => Key(Key::A.0 + (ch as i32 - 'A' as i32)),
            '-' | '_' => Key::MINUS,
            '.' | '>' => Key::PERIOD,
            '\'' | '"' => Key::QUOTE,
            ';' | ':' => Key::SEMICOLON,
            '/' | '?' => Key::SLASH,
            ' ' => Key::SPACE,
            _ => Key::UNIDENTIFIED,
        }
    }

    pub fn function(number: u8) -> Option<Key> {
        (1..=12)
            .contains(&number)
            .then(|| Key(Key::F1.0 + i32::from(number - 1)))
    }

    /// The function-key number this key is, the inverse of [`Key::function`].
    pub(crate) fn function_number(self) -> Option<u8> {
        let offset = self.0 - Key::F1.0;
        (0..12)
            .contains(&offset)
            .then(|| u8::try_from(offset + 1).unwrap_or(1))
    }

    pub fn numpad_digit(digit: char) -> Option<Key> {
        digit
            .to_digit(10)
            .map(|digit| Key(Key::NUMPAD_0.0 + digit as i32))
    }
}

/// One key press, as the encoder sees it.
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent<'a> {
    pub key: Key,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    /// The text the press would insert, when it inserts any.
    pub text: Option<&'a str>,
    /// The character the key bears with no shift applied, stated separately
    /// from the key for a caller that already knows it.
    pub unshifted_codepoint: Option<char>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
}

/// One cell-addressed mouse event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub button: Option<MouseButton>,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub column: u16,
    pub row: u16,
    /// Whether any button is currently down, which the motion modes need at
    /// viewport edges.
    pub any_button_pressed: bool,
}

/// The bytes this key press produces on a terminal in its default modes —
/// what `send-keys` needs for a client, where there is no pane whose modes
/// could apply.
pub fn encode_key_default_modes(key: KeyEvent<'_>) -> Vec<u8> {
    use super::screen::mode;
    super::engine::keys::encode_key(mode::CURSOR | mode::WRAP, key)
}

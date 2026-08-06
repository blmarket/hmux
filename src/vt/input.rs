//! The input-encoding seam: turning a key press or a mouse action into the
//! bytes the pane's program expects.
//!
//! This is deliberately separate from [`super::screen::VtScreen`]. Encoding
//! *reads* screen state — the cursor-key mode, the keypad mode, which mouse
//! protocol the program asked for — but it produces bytes for the child, not
//! cells for the grid, and it changes for different reasons. One god-trait over
//! both would tie a rewrite of either to the other.

use std::io;

/// A physical key identity, independent of the layout that produced it.
///
/// The code is opaque above the seam: only the constants below and
/// [`Key::from_ascii`], which maps a US-layout character onto the key that
/// bears it, name a key. A backend translates the code into whatever its own
/// encoder wants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Key(i32);

impl Key {
    /// The opaque code, for a backend that has to translate it. Only the
    /// libghostty-vt backend does; the engine reads the key itself.
    #[cfg_attr(not(feature = "ghostty"), allow(dead_code))]
    pub(crate) fn code(self) -> i32 {
        self.0
    }
}

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
    pub(crate) const BACKSPACE: Key = Key(53);
    pub(crate) const ENTER: Key = Key(58);
    pub(crate) const SPACE: Key = Key(63);
    pub(crate) const TAB: Key = Key(64);
    pub(crate) const DELETE: Key = Key(68);
    pub(crate) const END: Key = Key(69);
    pub(crate) const HOME: Key = Key(71);
    pub(crate) const INSERT: Key = Key(72);
    pub(crate) const PAGE_DOWN: Key = Key(73);
    pub(crate) const PAGE_UP: Key = Key(74);
    pub(crate) const ARROW_DOWN: Key = Key(75);
    pub(crate) const ARROW_LEFT: Key = Key(76);
    pub(crate) const ARROW_RIGHT: Key = Key(77);
    pub(crate) const ARROW_UP: Key = Key(78);
    pub(crate) const NUMPAD_0: Key = Key(80);
    pub(crate) const NUMPAD_ADD: Key = Key(90);
    pub(crate) const NUMPAD_DECIMAL: Key = Key(95);
    pub(crate) const NUMPAD_DIVIDE: Key = Key(96);
    pub(crate) const NUMPAD_ENTER: Key = Key(97);
    pub(crate) const NUMPAD_MULTIPLY: Key = Key(104);
    pub(crate) const NUMPAD_SUBTRACT: Key = Key(107);
    pub(crate) const ESCAPE: Key = Key(120);
    pub(crate) const F1: Key = Key(121);

    /// Map a printable US-layout ASCII character to the key that bears it.
    pub(crate) fn from_ascii(ch: char) -> Key {
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

    pub(crate) fn function(number: u8) -> Option<Key> {
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

    pub(crate) fn numpad_digit(digit: char) -> Option<Key> {
        digit
            .to_digit(10)
            .map(|digit| Key(Key::NUMPAD_0.0 + digit as i32))
    }
}

/// One key press, as the encoder sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct KeyEvent<'a> {
    pub(crate) key: Key,
    pub(crate) shift: bool,
    pub(crate) control: bool,
    pub(crate) alt: bool,
    /// The text the press would insert, when it inserts any.
    pub(crate) text: Option<&'a str>,
    /// The character the key bears with no shift applied. Only the
    /// libghostty-vt backend needs it told to it separately.
    #[cfg_attr(not(feature = "ghostty"), allow(dead_code))]
    pub(crate) unshifted_codepoint: Option<char>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseAction {
    Press,
    Release,
    Motion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseButton {
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
pub(crate) struct MouseEvent {
    pub(crate) action: MouseAction,
    pub(crate) button: Option<MouseButton>,
    pub(crate) shift: bool,
    pub(crate) control: bool,
    pub(crate) alt: bool,
    pub(crate) column: u16,
    pub(crate) row: u16,
    /// Whether any button is currently down, which the motion modes need at
    /// viewport edges.
    pub(crate) any_button_pressed: bool,
}

/// Encoding a press or a click for the program running in the pane.
///
/// The encoding depends on modes the program set, so this is implemented
/// alongside the screen that tracks them rather than as a free function.
pub(crate) trait InputEncoder {
    /// The bytes this key press produces under the screen's current modes.
    fn encode_key(&self, key: KeyEvent<'_>) -> io::Result<Vec<u8>>;

    /// The bytes this mouse event produces, or none when the program has asked
    /// for no mouse reports.
    fn encode_mouse(&self, mouse: MouseEvent) -> io::Result<Vec<u8>>;
}

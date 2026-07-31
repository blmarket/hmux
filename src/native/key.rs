//! Tmux-compatible semantic key identity and key-name conversion.
//!
//! Names are a control-plane representation. Terminal input decoding and pane
//! output encoding deliberately live elsewhere because both are stateful.

use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Modifiers(u8);

impl Modifiers {
    const META: u8 = 1;
    const CTRL: u8 = 2;
    const SHIFT: u8 = 4;

    pub(crate) fn meta(self) -> bool {
        self.0 & Self::META != 0
    }

    pub(crate) fn ctrl(self) -> bool {
        self.0 & Self::CTRL != 0
    }

    pub(crate) fn shift(self) -> bool {
        self.0 & Self::SHIFT != 0
    }

    pub(crate) fn new(meta: bool, ctrl: bool, shift: bool) -> Self {
        let mut modifiers = Self::default();
        if meta {
            modifiers.insert(Self::META);
        }
        if ctrl {
            modifiers.insert(Self::CTRL);
        }
        if shift {
            modifiers.insert(Self::SHIFT);
        }
        modifiers
    }

    fn insert(&mut self, bit: u8) {
        self.0 |= bit;
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SpecialKey {
    F(u8),
    Insert,
    Delete,
    Home,
    End,
    PageDown,
    PageUp,
    BackTab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Keypad(char),
    KeypadEnter,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MouseKind {
    Move,
    Down,
    Up,
    Drag,
    DragEnd,
    WheelUp,
    WheelDown,
    SecondClick,
    DoubleClick,
    TripleClick,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MouseLocation {
    Pane,
    Status,
    StatusLeft,
    StatusRight,
    StatusDefault,
    Border,
    ScrollbarUp,
    ScrollbarSlider,
    ScrollbarDown,
    Control(u8),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MouseKey {
    pub(crate) kind: MouseKind,
    /// Zero is tmux's unnumbered mouse form.
    pub(crate) button: u8,
    pub(crate) location: MouseLocation,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum KeyBase {
    Char(char),
    Special(SpecialKey),
    User(u16),
    Mouse(MouseKey),
    Any,
    None,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct KeyCode {
    pub(crate) base: KeyBase,
    pub(crate) modifiers: Modifiers,
}

impl KeyCode {
    pub(crate) fn new(base: KeyBase, modifiers: Modifiers) -> Self {
        Self { base, modifiers }
    }

    pub(crate) fn is_bindable(self) -> bool {
        !matches!(self.base, KeyBase::None)
    }
}

/// Convert one decoded C0/ASCII tty byte into tmux's semantic key identity.
pub(crate) fn key_from_byte(byte: u8) -> KeyCode {
    let mut modifiers = Modifiers::default();
    let base = match byte {
        0 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char(' ')
        }
        1..=8 | 10..=12 | 14..=26 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char(char::from(b'a' + byte - 1))
        }
        9 => KeyBase::Char('\t'),
        13 => KeyBase::Char('\r'),
        27 => KeyBase::Char('\u{1b}'),
        28 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char('\\')
        }
        29 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char(']')
        }
        30 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char('^')
        }
        31 => {
            modifiers.insert(Modifiers::CTRL);
            KeyBase::Char('_')
        }
        127 => KeyBase::Special(SpecialKey::Backspace),
        _ => KeyBase::Char(char::from(byte)),
    };
    KeyCode::new(base, modifiers)
}

/// Encode the byte-valued subset used by the `prefix` and `prefix2` options.
///
/// Stateful pane encoding is intentionally separate and handled by the
/// libghostty-vt-backed encoder.
pub(crate) fn basic_key_bytes(key: KeyCode) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(2);
    if key.modifiers.meta() {
        bytes.push(0x1b);
    }
    let byte = match key.base {
        KeyBase::Char(' ') if key.modifiers.ctrl() => 0,
        KeyBase::Char(ch) if key.modifiers.ctrl() && ch.is_ascii() => {
            (ch.to_ascii_lowercase() as u8) & 0x1f
        }
        KeyBase::Char(ch) if ch.is_ascii() => {
            let ch = if key.modifiers.shift() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            ch as u8
        }
        KeyBase::Special(SpecialKey::Backspace) => 127,
        _ => return None,
    };
    bytes.push(byte);
    Some(bytes)
}

/// Parse a tmux key name. Aliases resolve to one semantic identity.
pub(crate) fn parse_key_name(input: &str) -> Option<KeyCode> {
    if input.eq_ignore_ascii_case("None") {
        return Some(KeyCode::new(KeyBase::None, Modifiers::default()));
    }
    if input.eq_ignore_ascii_case("Any") {
        return Some(KeyCode::new(KeyBase::Any, Modifiers::default()));
    }

    // tmux accepts the numeric prefix parsed by sscanf("%x"), including
    // trailing non-hex text. Preserve that observable edge case.
    if let Some(hex) = input.strip_prefix("0x") {
        let digits = hex
            .as_bytes()
            .iter()
            .take_while(|byte| byte.is_ascii_hexdigit())
            .count();
        if digits == 0 {
            return None;
        }
        let value = u32::from_str_radix(&hex[..digits], 16).ok()?;
        let ch = char::from_u32(value)?;
        return Some(KeyCode::new(KeyBase::Char(ch), Modifiers::default()));
    }

    let mut modifiers = Modifiers::default();
    let mut rest = input;
    if rest.starts_with('^') && rest.chars().count() == 2 {
        modifiers.insert(Modifiers::CTRL);
        let ch = rest.chars().nth(1)?.to_ascii_lowercase();
        return Some(KeyCode::new(KeyBase::Char(ch), modifiers));
    }
    if rest.starts_with('^') {
        modifiers.insert(Modifiers::CTRL);
        rest = &rest[1..];
    }
    while rest.len() >= 2 && rest.as_bytes()[1] == b'-' {
        let bit = match rest.as_bytes()[0].to_ascii_lowercase() {
            b'c' => Modifiers::CTRL,
            b'm' => Modifiers::META,
            b's' => Modifiers::SHIFT,
            _ => return None,
        };
        modifiers.insert(bit);
        rest = &rest[2..];
    }
    if rest.is_empty() {
        return None;
    }

    let mut chars = rest.chars();
    if let (Some(ch), None) = (chars.next(), chars.next()) {
        if ch.is_ascii_control() {
            return None;
        }
        return Some(KeyCode::new(KeyBase::Char(ch), modifiers));
    }

    let base = parse_named_key(rest)?;
    Some(KeyCode::new(base, modifiers))
}

fn parse_named_key(name: &str) -> Option<KeyBase> {
    let lower = name.to_ascii_lowercase();
    let base = match lower.as_str() {
        "f1" => KeyBase::Special(SpecialKey::F(1)),
        "f2" => KeyBase::Special(SpecialKey::F(2)),
        "f3" => KeyBase::Special(SpecialKey::F(3)),
        "f4" => KeyBase::Special(SpecialKey::F(4)),
        "f5" => KeyBase::Special(SpecialKey::F(5)),
        "f6" => KeyBase::Special(SpecialKey::F(6)),
        "f7" => KeyBase::Special(SpecialKey::F(7)),
        "f8" => KeyBase::Special(SpecialKey::F(8)),
        "f9" => KeyBase::Special(SpecialKey::F(9)),
        "f10" => KeyBase::Special(SpecialKey::F(10)),
        "f11" => KeyBase::Special(SpecialKey::F(11)),
        "f12" => KeyBase::Special(SpecialKey::F(12)),
        "ic" | "insert" => KeyBase::Special(SpecialKey::Insert),
        "dc" | "delete" => KeyBase::Special(SpecialKey::Delete),
        "home" => KeyBase::Special(SpecialKey::Home),
        "end" => KeyBase::Special(SpecialKey::End),
        "npage" | "pagedown" | "pgdn" => KeyBase::Special(SpecialKey::PageDown),
        "ppage" | "pageup" | "pgup" => KeyBase::Special(SpecialKey::PageUp),
        "btab" => KeyBase::Special(SpecialKey::BackTab),
        "space" => KeyBase::Char(' '),
        "bspace" => KeyBase::Special(SpecialKey::Backspace),
        "tab" => KeyBase::Char('\t'),
        "enter" => KeyBase::Char('\r'),
        "escape" => KeyBase::Char('\u{1b}'),
        "up" => KeyBase::Special(SpecialKey::Up),
        "down" => KeyBase::Special(SpecialKey::Down),
        "left" => KeyBase::Special(SpecialKey::Left),
        "right" => KeyBase::Special(SpecialKey::Right),
        "kp/" => KeyBase::Special(SpecialKey::Keypad('/')),
        "kp*" => KeyBase::Special(SpecialKey::Keypad('*')),
        "kp-" => KeyBase::Special(SpecialKey::Keypad('-')),
        "kp+" => KeyBase::Special(SpecialKey::Keypad('+')),
        "kp." => KeyBase::Special(SpecialKey::Keypad('.')),
        "kp0" => KeyBase::Special(SpecialKey::Keypad('0')),
        "kp1" => KeyBase::Special(SpecialKey::Keypad('1')),
        "kp2" => KeyBase::Special(SpecialKey::Keypad('2')),
        "kp3" => KeyBase::Special(SpecialKey::Keypad('3')),
        "kp4" => KeyBase::Special(SpecialKey::Keypad('4')),
        "kp5" => KeyBase::Special(SpecialKey::Keypad('5')),
        "kp6" => KeyBase::Special(SpecialKey::Keypad('6')),
        "kp7" => KeyBase::Special(SpecialKey::Keypad('7')),
        "kp8" => KeyBase::Special(SpecialKey::Keypad('8')),
        "kp9" => KeyBase::Special(SpecialKey::Keypad('9')),
        "kpenter" => KeyBase::Special(SpecialKey::KeypadEnter),
        _ => {
            if let Some(user) = parse_user_key(&lower) {
                KeyBase::User(user)
            } else {
                KeyBase::Mouse(parse_mouse_key(name)?)
            }
        }
    };
    Some(base)
}

fn parse_user_key(lower: &str) -> Option<u16> {
    let suffix = lower.strip_prefix("user")?;
    let digits = suffix
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }
    let value = suffix[..digits].parse::<u16>().ok()?;
    (value <= 1000).then_some(value)
}

fn parse_mouse_key(name: &str) -> Option<MouseKey> {
    let lower = name.to_ascii_lowercase();
    let (kind, rest) = [
        ("mousemove", MouseKind::Move),
        ("mousedragend", MouseKind::DragEnd),
        ("secondclick", MouseKind::SecondClick),
        ("doubleclick", MouseKind::DoubleClick),
        ("tripleclick", MouseKind::TripleClick),
        ("mousedown", MouseKind::Down),
        ("mouseup", MouseKind::Up),
        ("mousedrag", MouseKind::Drag),
        ("wheeldown", MouseKind::WheelDown),
        ("wheelup", MouseKind::WheelUp),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| lower.strip_prefix(prefix).map(|rest| (kind, rest)))?;

    let button_len = rest
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    let button = if button_len == 0 {
        0
    } else {
        rest[..button_len].parse::<u8>().ok()?
    };
    if !matches!(button, 0 | 1 | 2 | 3 | 6..=11) {
        return None;
    }
    let location = match &rest[button_len..] {
        "pane" => MouseLocation::Pane,
        "status" => MouseLocation::Status,
        "statusleft" => MouseLocation::StatusLeft,
        "statusright" => MouseLocation::StatusRight,
        "statusdefault" => MouseLocation::StatusDefault,
        "border" => MouseLocation::Border,
        "scrollbarup" => MouseLocation::ScrollbarUp,
        "scrollbarslider" => MouseLocation::ScrollbarSlider,
        "scrollbardown" => MouseLocation::ScrollbarDown,
        value if value.starts_with("control") => {
            let control = value["control".len()..].parse::<u8>().ok()?;
            if control > 9 {
                return None;
            }
            MouseLocation::Control(control)
        }
        _ => return None,
    };
    Some(MouseKey {
        kind,
        button,
        location,
    })
}

/// Return tmux's canonical spelling for a semantic key.
pub(crate) fn format_key_name(key: KeyCode) -> String {
    let mut out = String::new();
    if key.modifiers.ctrl() {
        out.push_str("C-");
    }
    if key.modifiers.meta() {
        out.push_str("M-");
    }
    if key.modifiers.shift() {
        out.push_str("S-");
    }
    match key.base {
        KeyBase::Char(ch) => format_char(ch, &mut out),
        KeyBase::Special(special) => out.push_str(special_name(special)),
        KeyBase::User(number) => out.push_str(&format!("User{number}")),
        KeyBase::Mouse(mouse) => format_mouse(mouse, &mut out),
        KeyBase::Any => out.push_str("Any"),
        KeyBase::None => out.push_str("None"),
    }
    out
}

fn format_char(ch: char, out: &mut String) {
    match ch {
        ' ' => out.push_str("Space"),
        '\t' => out.push_str("Tab"),
        '\r' => out.push_str("Enter"),
        '\u{1b}' => out.push_str("Escape"),
        ch if ch.is_control() => out.push_str(&format!("0x{:x}", ch as u32)),
        ch => out.push(ch),
    }
}

fn special_name(key: SpecialKey) -> &'static str {
    match key {
        SpecialKey::F(1) => "F1",
        SpecialKey::F(2) => "F2",
        SpecialKey::F(3) => "F3",
        SpecialKey::F(4) => "F4",
        SpecialKey::F(5) => "F5",
        SpecialKey::F(6) => "F6",
        SpecialKey::F(7) => "F7",
        SpecialKey::F(8) => "F8",
        SpecialKey::F(9) => "F9",
        SpecialKey::F(10) => "F10",
        SpecialKey::F(11) => "F11",
        SpecialKey::F(12) => "F12",
        SpecialKey::F(_) => "Unknown",
        SpecialKey::Insert => "IC",
        SpecialKey::Delete => "DC",
        SpecialKey::Home => "Home",
        SpecialKey::End => "End",
        SpecialKey::PageDown => "NPage",
        SpecialKey::PageUp => "PPage",
        SpecialKey::BackTab => "BTab",
        SpecialKey::Backspace => "BSpace",
        SpecialKey::Up => "Up",
        SpecialKey::Down => "Down",
        SpecialKey::Left => "Left",
        SpecialKey::Right => "Right",
        SpecialKey::Keypad('/') => "KP/",
        SpecialKey::Keypad('*') => "KP*",
        SpecialKey::Keypad('-') => "KP-",
        SpecialKey::Keypad('+') => "KP+",
        SpecialKey::Keypad('.') => "KP.",
        SpecialKey::Keypad('0') => "KP0",
        SpecialKey::Keypad('1') => "KP1",
        SpecialKey::Keypad('2') => "KP2",
        SpecialKey::Keypad('3') => "KP3",
        SpecialKey::Keypad('4') => "KP4",
        SpecialKey::Keypad('5') => "KP5",
        SpecialKey::Keypad('6') => "KP6",
        SpecialKey::Keypad('7') => "KP7",
        SpecialKey::Keypad('8') => "KP8",
        SpecialKey::Keypad('9') => "KP9",
        SpecialKey::Keypad(_) => "Unknown",
        SpecialKey::KeypadEnter => "KPEnter",
    }
}

fn format_mouse(mouse: MouseKey, out: &mut String) {
    out.push_str(match mouse.kind {
        MouseKind::Move => "MouseMove",
        MouseKind::Down => "MouseDown",
        MouseKind::Up => "MouseUp",
        MouseKind::Drag => "MouseDrag",
        MouseKind::DragEnd => "MouseDragEnd",
        MouseKind::WheelUp => "WheelUp",
        MouseKind::WheelDown => "WheelDown",
        MouseKind::SecondClick => "SecondClick",
        MouseKind::DoubleClick => "DoubleClick",
        MouseKind::TripleClick => "TripleClick",
    });
    if mouse.button != 0 {
        out.push_str(&mouse.button.to_string());
    }
    match mouse.location {
        MouseLocation::Pane => out.push_str("Pane"),
        MouseLocation::Status => out.push_str("Status"),
        MouseLocation::StatusLeft => out.push_str("StatusLeft"),
        MouseLocation::StatusRight => out.push_str("StatusRight"),
        MouseLocation::StatusDefault => out.push_str("StatusDefault"),
        MouseLocation::Border => out.push_str("Border"),
        MouseLocation::ScrollbarUp => out.push_str("ScrollbarUp"),
        MouseLocation::ScrollbarSlider => out.push_str("ScrollbarSlider"),
        MouseLocation::ScrollbarDown => out.push_str("ScrollbarDown"),
        MouseLocation::Control(control) => out.push_str(&format!("Control{control}")),
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_key_name(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_have_one_identity_and_canonical_name() {
        let canonical = parse_key_name("PPage").unwrap();
        assert_eq!(parse_key_name("PageUp"), Some(canonical));
        assert_eq!(parse_key_name("PgUp"), Some(canonical));
        assert_eq!(format_key_name(canonical), "PPage");
    }

    #[test]
    fn parses_modifiers_unicode_user_and_mouse() {
        for name in [
            "C-M-S-Left",
            "é",
            "User1000",
            "MouseMovePane",
            "MouseDown1Pane",
            "WheelUpScrollbarSlider",
        ] {
            let key = parse_key_name(name).unwrap();
            assert_eq!(parse_key_name(&format_key_name(key)), Some(key));
        }
    }

    #[test]
    fn rejects_unknown_and_out_of_range_names() {
        assert_eq!(parse_key_name("DefinitelyNotAKey"), None);
        assert_eq!(parse_key_name("User1001"), None);
        assert_eq!(parse_key_name("MouseDown4Pane"), None);
    }
}

//! Encoding a semantic key into the bytes a pane's program expects.
//!
//! This is a port of tmux's `input-keys.c`, not a use of the terminal engine's
//! key encoder. The two disagree, and the disagreement is not cosmetic: the
//! engine encodes for a terminal that advertises modern capabilities, while a
//! pane is told it is running under `screen`/`tmux-256color`. Anything the
//! engine emits beyond what that terminfo describes — `CSI H` for `Home`,
//! `CSI 105;5u` for `C-i`, `CSI 13;5~` for `C-F3` — reaches a program that has
//! no way to recognize it. `less` reading `CSI F` for `End` is the canonical
//! symptom: it drops the introducer and runs the trailing `F` as its own
//! "forward forever" command.
//!
//! So the pane direction follows tmux's table and its downgrade rules, and the
//! engine keeps the output direction it is authoritative for. See README.md.

use super::key::{KeyBase, KeyCode, Modifiers, SpecialKey};

/// A key on its way to a pane, with the terminal-shaped flags tmux carries
/// alongside the key code itself.
///
/// tmux packs these into spare bits of `key_code`; hmux keeps `KeyCode` a pure
/// semantic identity (it is also the key-table lookup key, where the flags must
/// not participate) and carries them beside it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneKey {
    pub(crate) code: KeyCode,
    /// tmux's `KEYC_CURSOR`: this key names, or arrived as, the application
    /// cursor form (`SS3 A`) rather than the normal one (`CSI A`).
    pub(crate) cursor: bool,
    /// tmux's `KEYC_KEYPAD`: likewise for the application keypad forms.
    pub(crate) keypad: bool,
    /// tmux's `KEYC_IMPLIED_META`: the meta modifier is already carried inside
    /// the sequence this key stands for, so no `ESC` prefix is added for it.
    pub(crate) implied_meta: bool,
}

impl PaneKey {
    /// The flags tmux's `key_string_table` attaches to a key *name*.
    ///
    /// Cursor and keypad keys are named with their application form implied;
    /// `input_key` strips the flag again when the pane is not in that mode.
    /// `KEYC_IMPLIED_META` is attached to the same special keys but then
    /// cleared unless the name was written with `M-`, because only then does a
    /// table entry exist that already spells the modifier out.
    pub(crate) fn from_name(code: KeyCode) -> Self {
        let cursor = matches!(
            code.base,
            KeyBase::Special(
                SpecialKey::Up | SpecialKey::Down | SpecialKey::Left | SpecialKey::Right
            )
        );
        let keypad = matches!(
            code.base,
            KeyBase::Special(SpecialKey::Keypad(_) | SpecialKey::KeypadEnter)
        );
        let implied_meta = code.modifiers.meta() && named_key_implies_meta(code.base);
        Self {
            code,
            cursor,
            keypad,
            implied_meta,
        }
    }
}

/// Whether tmux's key-name table tags this key with `KEYC_IMPLIED_META`.
///
/// These are exactly the keys whose modified forms are built from
/// `KEYC_BUILD_MODIFIERS`, so a `M-` form has a table entry of its own and must
/// not also collect an `ESC` prefix.
fn named_key_implies_meta(base: KeyBase) -> bool {
    matches!(
        base,
        KeyBase::Special(
            SpecialKey::F(_)
                | SpecialKey::Insert
                | SpecialKey::Delete
                | SpecialKey::Home
                | SpecialKey::End
                | SpecialKey::PageDown
                | SpecialKey::PageUp
                | SpecialKey::Up
                | SpecialKey::Down
                | SpecialKey::Left
                | SpecialKey::Right
        )
    )
}

/// The pane's own terminal modes, as far as key output is concerned.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneKeyModes {
    /// DECCKM (`CSI ? 1 h`), tmux's `MODE_KCURSOR`.
    pub(crate) cursor_keys: bool,
    /// DECKPAM (`ESC =`), tmux's `MODE_KKEYPAD`.
    pub(crate) application_keypad: bool,
    /// The effective `modifyOtherKeys` state, after the `extended-keys` option
    /// has been applied to what the pane asked for.
    pub(crate) extended: ExtendedKeys,
    /// DECSET 2004, tmux's `MODE_BRACKETPASTE`: whether the pane wants the
    /// paste markers at all.
    pub(crate) bracketed_paste: bool,
}

/// How much of the keyboard a pane asked to receive in extended form.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ExtendedKeys {
    /// tmux's "standard" output: the VT10x downgrade, which drops modifiers it
    /// cannot express. This is the default, and the only state most programs
    /// running under `TERM=tmux-256color` can make sense of.
    #[default]
    Off,
    /// `modifyOtherKeys=1` (tmux's `MODE_KEYS_EXTENDED`): keys that a vt10x
    /// terminal could have produced still go out in the standard form.
    Standard,
    /// `modifyOtherKeys=2` (tmux's `MODE_KEYS_EXTENDED_2`): every modified key
    /// goes out in extended form.
    All,
}

/// The wire form used for extended keys, from `extended-keys-format`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ExtendedKeysFormat {
    /// `csi-u`: `CSI <key> ; <modifier> u`.
    CsiU,
    /// `xterm`: `CSI 27 ; <modifier> ; <key> ~`. tmux's default.
    #[default]
    Xterm,
}

/// Server options that change how a key is encoded.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PaneKeyOptions {
    /// The `backspace` option: what `BSpace` is actually sent as.
    pub(crate) backspace: KeyCode,
    pub(crate) extended_keys_format: ExtendedKeysFormat,
}

impl Default for PaneKeyOptions {
    fn default() -> Self {
        Self {
            // tmux's default is the bare `DEL` code, which it *displays* as
            // `C-?` because that is how it renders any control byte. The two
            // are not the same key: `C-?` parses back to `'?'` with control,
            // which is why the option is read through
            // `ServerState::pane_key_options` rather than `parse_key_name`
            // alone.
            backspace: KeyCode::new(KeyBase::Char('\u{7f}'), Modifiers::default()),
            extended_keys_format: ExtendedKeysFormat::default(),
        }
    }
}

/// What a key turned into on its way to a pane.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneKeyEncoding {
    /// The bytes to write, which may be empty for a key tmux deliberately
    /// swallows — a paste marker outside bracketed-paste mode, say.
    pub(crate) bytes: Vec<u8>,
    /// tmux's `input_key` returning 0 rather than -1. A false here means the
    /// key has no form this pane could receive, which `send-keys` turns into
    /// its literal-string fallback; any bytes above were still written, as
    /// they are by tmux, which emits the meta prefix before it gives up.
    pub(crate) complete: bool,
}

impl PaneKeyEncoding {
    fn encoded(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            complete: true,
        }
    }

    fn handled() -> Self {
        Self::encoded(Vec::new())
    }

    fn unencodable(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            complete: false,
        }
    }
}

/// Encode one key for a pane.
pub(crate) fn encode(
    key: PaneKey,
    modes: PaneKeyModes,
    options: PaneKeyOptions,
) -> PaneKeyEncoding {
    let PaneKey {
        mut code,
        mut cursor,
        mut keypad,
        mut implied_meta,
    } = key;

    if matches!(
        code.base,
        KeyBase::Mouse(_) | KeyBase::Any | KeyBase::None | KeyBase::User(_)
    ) {
        return PaneKeyEncoding::handled();
    }

    // tmux's `input_key` answers the paste markers before anything else, and
    // writes them only to a pane that asked for bracketed paste.
    if let KeyBase::Special(marker @ (SpecialKey::PasteStart | SpecialKey::PasteEnd)) = code.base {
        if !modes.bracketed_paste {
            return PaneKeyEncoding::handled();
        }
        return PaneKeyEncoding::encoded(
            match marker {
                SpecialKey::PasteStart => b"\x1b[200~",
                _ => b"\x1b[201~",
            }
            .to_vec(),
        );
    }

    // Backspace is whatever the `backspace` option says it is, which is how
    // tmux lets a terminal whose `kbs` is `^H` reach programs unmangled.
    if code.base == KeyBase::Special(SpecialKey::Backspace) {
        let replacement = options.backspace;
        if code.modifiers == Modifiers::default() {
            return PaneKeyEncoding::encoded(
                backspace_byte(replacement).map_or_else(Vec::new, |byte| vec![byte]),
            );
        }
        code = KeyCode::new(
            replacement.base,
            union_modifiers(replacement.modifiers, code.modifiers),
        );
    }

    // Backtab has no modified forms of its own: extended mode re-spells it as
    // `S-Tab`, and everything else drops the modifiers and sends bare `CSI Z`.
    if code.base == KeyBase::Special(SpecialKey::BackTab) {
        if modes.extended == ExtendedKeys::All {
            code = KeyCode::new(
                KeyBase::Char('\t'),
                union_modifiers(code.modifiers, Modifiers::new(false, false, true)),
            );
        } else {
            code = KeyCode::new(code.base, Modifiers::default());
            implied_meta = false;
        }
    }

    // A bare 7-bit key that a keyboard can actually produce goes as itself.
    if code.modifiers == Modifiers::default() && !cursor && !keypad && !implied_meta {
        if let KeyBase::Char(ch) = code.base {
            if matches!(ch, '\t' | '\r' | '\u{1b}' | ' '..='\u{7f}') {
                return PaneKeyEncoding::encoded(vec![ch as u8]);
            }
            if !ch.is_ascii() {
                return PaneKeyEncoding::encoded(ch.to_string().into_bytes());
            }
        }
    }

    // Application-form entries only apply while the pane is in that mode.
    if !modes.application_keypad {
        keypad = false;
    }
    if !modes.cursor_keys {
        cursor = false;
    }

    // Meta that is not already spelled out inside the sequence becomes an
    // `ESC` prefix, and the lookup proceeds without it.
    let meta_prefix = code.modifiers.meta() && !implied_meta;
    let lookup = if meta_prefix {
        code.modifiers.without_meta()
    } else {
        code.modifiers
    };
    let mut found = table_entry(code.base, lookup, cursor, keypad);
    if found.is_none() && cursor {
        found = table_entry(code.base, lookup, false, keypad);
    }
    if found.is_none() && keypad {
        found = table_entry(code.base, lookup, cursor, false);
    }
    if let Some(data) = found {
        let mut out = Vec::with_capacity(data.len() + 1);
        if meta_prefix {
            out.push(0x1b);
        }
        out.extend_from_slice(&data);
        return PaneKeyEncoding::encoded(out);
    }

    // Every remaining special key is one of tmux's internal codes, which have
    // no pane representation at all.
    if matches!(code.base, KeyBase::Special(_)) {
        return PaneKeyEncoding::handled();
    }

    let format = options.extended_keys_format;
    match modes.extended {
        ExtendedKeys::All => append_extended(Vec::new(), code, format),
        ExtendedKeys::Standard => {
            let attempt = mode1(code);
            if attempt.complete {
                attempt
            } else {
                append_extended(attempt.bytes, code, format)
            }
        }
        ExtendedKeys::Off => vt10x(code),
    }
}

/// Fall through to the extended form, keeping anything a failed standard-mode
/// attempt already wrote.
fn append_extended(
    mut bytes: Vec<u8>,
    code: KeyCode,
    format: ExtendedKeysFormat,
) -> PaneKeyEncoding {
    match extended(code, format) {
        Some(tail) => {
            bytes.extend_from_slice(&tail);
            PaneKeyEncoding::encoded(bytes)
        }
        None => PaneKeyEncoding::unencodable(bytes),
    }
}

fn union_modifiers(left: Modifiers, right: Modifiers) -> Modifiers {
    Modifiers::new(
        left.meta() || right.meta(),
        left.ctrl() || right.ctrl(),
        left.shift() || right.shift(),
    )
}

/// The single byte an unmodified `BSpace` becomes, per the `backspace` option.
///
/// tmux accepts a plain key or a control key there and silently sends nothing
/// for anything else.
fn backspace_byte(replacement: KeyCode) -> Option<u8> {
    let KeyBase::Char(ch) = replacement.base else {
        // `backspace BSpace` and the like: tmux stores the key code, and only a
        // plain or control character survives the conversion below.
        return (replacement.base == KeyBase::Special(SpecialKey::Backspace)).then_some(0x7f);
    };
    if replacement.modifiers == Modifiers::default() {
        return ch.is_ascii().then_some(ch as u8);
    }
    if replacement.modifiers != Modifiers::new(false, true, false) {
        return None;
    }
    match ch {
        '?' => Some(0x7f),
        '@'..='_' => Some(ch as u8 - 0x40),
        'a'..='z' => Some(ch as u8 - 0x60),
        _ => None,
    }
}

/// tmux's `input_key_defaults` table, including the entries it builds for the
/// modified forms from `KEYC_BUILD_MODIFIERS`.
fn table_entry(base: KeyBase, modifiers: Modifiers, cursor: bool, keypad: bool) -> Option<Vec<u8>> {
    let KeyBase::Special(special) = base else {
        return None;
    };
    if modifiers == Modifiers::default() {
        return plain_entry(special, cursor, keypad).map(<[u8]>::to_vec);
    }
    // Only the built forms carry modifiers, and every built form that includes
    // meta also implies it, so a caller that stripped an `ESC` prefix has
    // already cleared meta here.
    let modifier = modifier_parameter(modifiers)?;
    let (prefix, final_byte) = modified_entry(special)?;
    let mut out = Vec::with_capacity(prefix.len() + 2);
    out.extend_from_slice(prefix);
    out.push(b'0' + modifier);
    out.push(final_byte);
    Some(out)
}

/// tmux's `input_key_modifiers` index for a modifier set, 2..=8.
fn modifier_parameter(modifiers: Modifiers) -> Option<u8> {
    let value = 1
        + u8::from(modifiers.shift())
        + 2 * u8::from(modifiers.meta())
        + 4 * u8::from(modifiers.ctrl());
    (value >= 2).then_some(value)
}

fn plain_entry(special: SpecialKey, cursor: bool, keypad: bool) -> Option<&'static [u8]> {
    let entry: &'static [u8] = match special {
        // Answered before this table, and never through the plain forms.
        SpecialKey::PasteStart | SpecialKey::PasteEnd => return None,
        SpecialKey::F(1) => b"\x1bOP",
        SpecialKey::F(2) => b"\x1bOQ",
        SpecialKey::F(3) => b"\x1bOR",
        SpecialKey::F(4) => b"\x1bOS",
        SpecialKey::F(5) => b"\x1b[15~",
        SpecialKey::F(6) => b"\x1b[17~",
        SpecialKey::F(7) => b"\x1b[18~",
        SpecialKey::F(8) => b"\x1b[19~",
        SpecialKey::F(9) => b"\x1b[20~",
        SpecialKey::F(10) => b"\x1b[21~",
        SpecialKey::F(11) => b"\x1b[23~",
        SpecialKey::F(12) => b"\x1b[24~",
        SpecialKey::F(_) => return None,
        SpecialKey::Insert => b"\x1b[2~",
        SpecialKey::Delete => b"\x1b[3~",
        // `khome`/`kend` of the terminfo the pane is given, not the `CSI H`/
        // `CSI F` a modern xterm reports.
        SpecialKey::Home => b"\x1b[1~",
        SpecialKey::End => b"\x1b[4~",
        SpecialKey::PageDown => b"\x1b[6~",
        SpecialKey::PageUp => b"\x1b[5~",
        SpecialKey::BackTab => b"\x1b[Z",
        SpecialKey::Up if cursor => b"\x1bOA",
        SpecialKey::Down if cursor => b"\x1bOB",
        SpecialKey::Right if cursor => b"\x1bOC",
        SpecialKey::Left if cursor => b"\x1bOD",
        SpecialKey::Up => b"\x1b[A",
        SpecialKey::Down => b"\x1b[B",
        SpecialKey::Right => b"\x1b[C",
        SpecialKey::Left => b"\x1b[D",
        SpecialKey::KeypadEnter if keypad => b"\x1bOM",
        SpecialKey::KeypadEnter => b"\n",
        SpecialKey::Keypad(digit) if keypad => keypad_application_entry(digit)?,
        SpecialKey::Keypad(digit) => return keypad_numeric_entry(digit),
        SpecialKey::Backspace => return None,
    };
    Some(entry)
}

fn keypad_application_entry(digit: char) -> Option<&'static [u8]> {
    let entry: &'static [u8] = match digit {
        '/' => b"\x1bOo",
        '*' => b"\x1bOj",
        '-' => b"\x1bOm",
        '+' => b"\x1bOk",
        '.' => b"\x1bOn",
        '0' => b"\x1bOp",
        '1' => b"\x1bOq",
        '2' => b"\x1bOr",
        '3' => b"\x1bOs",
        '4' => b"\x1bOt",
        '5' => b"\x1bOu",
        '6' => b"\x1bOv",
        '7' => b"\x1bOw",
        '8' => b"\x1bOx",
        '9' => b"\x1bOy",
        _ => return None,
    };
    Some(entry)
}

/// Outside application keypad mode a keypad key is simply its own character.
fn keypad_numeric_entry(digit: char) -> Option<&'static [u8]> {
    let entry: &'static [u8] = match digit {
        '/' => b"/",
        '*' => b"*",
        '-' => b"-",
        '+' => b"+",
        '.' => b".",
        '0' => b"0",
        '1' => b"1",
        '2' => b"2",
        '3' => b"3",
        '4' => b"4",
        '5' => b"5",
        '6' => b"6",
        '7' => b"7",
        '8' => b"8",
        '9' => b"9",
        _ => return None,
    };
    Some(entry)
}

/// The `(prefix, final byte)` halves of tmux's `KEYC_BUILD_MODIFIERS` entries,
/// with the modifier digit going between them.
fn modified_entry(special: SpecialKey) -> Option<(&'static [u8], u8)> {
    let entry: (&'static [u8], u8) = match special {
        // F1-F4 keep the `SS3` finals of their unmodified forms, which is what
        // `kf13`-`kf16` and `kf25`-`kf28` describe. Notably F3 is `CSI 1;<m>R`
        // and not the `CSI 13;<m>~` a modern terminal would report.
        SpecialKey::F(1) => (b"\x1b[1;", b'P'),
        SpecialKey::F(2) => (b"\x1b[1;", b'Q'),
        SpecialKey::F(3) => (b"\x1b[1;", b'R'),
        SpecialKey::F(4) => (b"\x1b[1;", b'S'),
        SpecialKey::F(5) => (b"\x1b[15;", b'~'),
        SpecialKey::F(6) => (b"\x1b[17;", b'~'),
        SpecialKey::F(7) => (b"\x1b[18;", b'~'),
        SpecialKey::F(8) => (b"\x1b[19;", b'~'),
        SpecialKey::F(9) => (b"\x1b[20;", b'~'),
        SpecialKey::F(10) => (b"\x1b[21;", b'~'),
        SpecialKey::F(11) => (b"\x1b[23;", b'~'),
        SpecialKey::F(12) => (b"\x1b[24;", b'~'),
        SpecialKey::Up => (b"\x1b[1;", b'A'),
        SpecialKey::Down => (b"\x1b[1;", b'B'),
        SpecialKey::Right => (b"\x1b[1;", b'C'),
        SpecialKey::Left => (b"\x1b[1;", b'D'),
        SpecialKey::Home => (b"\x1b[1;", b'H'),
        SpecialKey::End => (b"\x1b[1;", b'F'),
        SpecialKey::PageUp => (b"\x1b[5;", b'~'),
        SpecialKey::PageDown => (b"\x1b[6;", b'~'),
        SpecialKey::Insert => (b"\x1b[2;", b'~'),
        SpecialKey::Delete => (b"\x1b[3;", b'~'),
        _ => return None,
    };
    Some(entry)
}

/// tmux's `input_key_vt10x`: the standard mode, which remaps what it can into
/// the C0 and printable-ASCII forms a VT100-era keyboard would have produced
/// and loses whatever it cannot express.
/// The meta prefix goes out before the remapping is attempted, so a key that
/// turns out to have no standard form still leaves its `ESC` behind — as it
/// does in tmux, where `send-keys` then appends its literal fallback to it.
fn vt10x(code: KeyCode) -> PaneKeyEncoding {
    let mut out = Vec::with_capacity(2);
    if code.modifiers.meta() {
        out.push(0x1b);
    }
    let KeyBase::Char(ch) = code.base else {
        return PaneKeyEncoding::unencodable(out);
    };
    // Modifiers cannot be reported for a non-ASCII key in this mode, so they
    // are simply dropped.
    if !ch.is_ascii() {
        out.extend_from_slice(ch.to_string().as_bytes());
        return PaneKeyEncoding::encoded(out);
    }

    let only_key = ch as u8;
    // Tab, Enter and Escape must not be swallowed by the C0 remapping below.
    let ctrl = code.modifiers.ctrl() && !matches!(only_key, b'\r' | b'\n' | b'\t');

    // Shift is deliberately not handled: no terminal reports an unshifted key
    // with a shift modifier, only the shifted key itself.
    let byte = if ctrl {
        match STANDARD_MAP_FROM.iter().position(|&from| from == only_key) {
            Some(index) => STANDARD_MAP_TO[index],
            None if (b'3'..=b'7').contains(&only_key) => only_key - 0x18,
            None if (b'@'..=b'~').contains(&only_key) => only_key & 0x1f,
            None => return PaneKeyEncoding::unencodable(out),
        }
    } else {
        only_key
    };
    out.push(byte & 0x7f);
    PaneKeyEncoding::encoded(out)
}

/// tmux's `standard_map`: keys whose control form is a printable character or a
/// C0 code that does not follow the `& 0x1f` rule.
const STANDARD_MAP_FROM: &[u8; 22] = b"1!9(0)=+;:'\",<.>/-8? 2";
const STANDARD_MAP_TO: &[u8; 22] = b"119900=+;;'',,..\x1f\x1f\x7f\x7f\0\0";

/// tmux's `input_key_mode1`: under `modifyOtherKeys=1` the keys a vt10x
/// terminal could have produced keep their standard form.
fn mode1(code: KeyCode) -> PaneKeyEncoding {
    // A regular or shifted key plus meta only.
    if code.modifiers.meta() && !code.modifiers.ctrl() {
        return vt10x(code);
    }
    let KeyBase::Char(ch) = code.base else {
        return PaneKeyEncoding::unencodable(Vec::new());
    };
    // The set from https://invisible-island.net/xterm/modified-keys-us-pc105.html.
    let standard =
        ch.is_ascii() && matches!(ch as u8, b' ' | b'/' | b'@' | b'^' | b'2'..=b'8' | b'@'..=b'~');
    if code.modifiers.ctrl() && standard {
        return vt10x(code);
    }
    PaneKeyEncoding::unencodable(Vec::new())
}

/// tmux's `input_key_extended`: the `modifyOtherKeys` wire forms.
fn extended(code: KeyCode, format: ExtendedKeysFormat) -> Option<Vec<u8>> {
    let modifier = modifier_parameter(code.modifiers)?;
    let KeyBase::Char(ch) = code.base else {
        return None;
    };
    let key = ch as u32;
    let encoded = match format {
        ExtendedKeysFormat::CsiU => format!("\x1b[{key};{modifier}u"),
        ExtendedKeysFormat::Xterm => format!("\x1b[27;{modifier};{key}~"),
    };
    Some(encoded.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::key::parse_key_name;

    fn encoded(name: &str, modes: PaneKeyModes) -> PaneKeyEncoding {
        let code = parse_key_name(name).expect("key name parses");
        encode(PaneKey::from_name(code), modes, PaneKeyOptions::default())
    }

    /// The bytes of a key that encodes completely.
    fn bytes(name: &str) -> Vec<u8> {
        let encoding = encoded(name, PaneKeyModes::default());
        assert!(encoding.complete, "{name} should encode");
        encoding.bytes
    }

    #[test]
    fn home_and_end_use_the_terminfo_forms_not_the_xterm_ones() {
        assert_eq!(bytes("Home"), b"\x1b[1~");
        assert_eq!(bytes("End"), b"\x1b[4~");
        let application = PaneKeyModes {
            cursor_keys: true,
            ..PaneKeyModes::default()
        };
        assert_eq!(encoded("Home", application).bytes, b"\x1b[1~");
        assert_eq!(encoded("End", application).bytes, b"\x1b[4~");
    }

    #[test]
    fn arrows_follow_the_pane_application_cursor_mode() {
        assert_eq!(bytes("Up"), b"\x1b[A");
        let application = PaneKeyModes {
            cursor_keys: true,
            ..PaneKeyModes::default()
        };
        assert_eq!(encoded("Up", application).bytes, b"\x1bOA");
        // Modified arrows have one form either way.
        assert_eq!(bytes("S-Up"), b"\x1b[1;2A");
        assert_eq!(encoded("S-Up", application).bytes, b"\x1b[1;2A");
    }

    #[test]
    fn keypad_follows_the_pane_application_keypad_mode() {
        assert_eq!(bytes("KP1"), b"1");
        assert_eq!(bytes("KPEnter"), b"\n");
        let application = PaneKeyModes {
            application_keypad: true,
            ..PaneKeyModes::default()
        };
        assert_eq!(encoded("KP1", application).bytes, b"\x1bOq");
        assert_eq!(encoded("KPEnter", application).bytes, b"\x1bOM");
        assert_eq!(encoded("KP/", application).bytes, b"\x1bOo");
    }

    #[test]
    fn modified_function_keys_keep_their_terminfo_finals() {
        // `kf15`/`kf27` of `tmux-256color`, which a `CSI 13;<m>~` would miss.
        assert_eq!(bytes("S-F3"), b"\x1b[1;2R");
        assert_eq!(bytes("C-F3"), b"\x1b[1;5R");
        assert_eq!(bytes("S-F5"), b"\x1b[15;2~");
    }

    #[test]
    fn the_standard_mode_downgrades_instead_of_going_extended() {
        assert_eq!(bytes("C-i"), b"\t");
        assert_eq!(bytes("C-m"), b"\r");
        assert_eq!(bytes("C-Enter"), b"\r");
        assert_eq!(bytes("S-Enter"), b"\r");
        assert_eq!(bytes("C-Tab"), b"\t");
        assert_eq!(bytes("C-Space"), b"\0");
        assert_eq!(bytes("C-'"), b"'");
        assert_eq!(bytes("C-,"), b",");
        assert_eq!(bytes("C--"), b"\x1f");
        assert_eq!(bytes("C-["), b"\x1b");
        assert_eq!(bytes("C-`"), b"\0");
        assert_eq!(bytes("C-3"), b"\x1b");
        // Shift alone is not reportable, so it is dropped.
        assert_eq!(bytes("S-a"), b"a");
        assert_eq!(bytes("C-S-a"), b"\x01");
    }

    #[test]
    fn keys_with_no_standard_form_report_that_they_were_not_encoded() {
        assert!(!encoded("C-Escape", PaneKeyModes::default()).complete);
    }

    #[test]
    fn extended_mode_two_reports_every_modified_key() {
        let modes = PaneKeyModes {
            extended: ExtendedKeys::All,
            ..PaneKeyModes::default()
        };
        assert_eq!(encoded("C-Enter", modes).bytes, b"\x1b[27;5;13~");
        assert_eq!(encoded("C-Escape", modes).bytes, b"\x1b[27;5;27~");
        // Table entries still win over the extended form.
        assert_eq!(encoded("S-Up", modes).bytes, b"\x1b[1;2A");
        assert_eq!(encoded("Home", modes).bytes, b"\x1b[1~");
        // Backtab becomes S-Tab rather than losing its modifiers.
        assert_eq!(encoded("C-BTab", modes).bytes, b"\x1b[27;6;9~");
    }

    #[test]
    fn extended_mode_one_keeps_the_standard_form_where_one_exists() {
        let modes = PaneKeyModes {
            extended: ExtendedKeys::Standard,
            ..PaneKeyModes::default()
        };
        assert_eq!(encoded("C-a", modes).bytes, b"\x01");
        assert_eq!(encoded("M-a", modes).bytes, b"\x1ba");
        assert_eq!(encoded("C-Enter", modes).bytes, b"\x1b[27;5;13~");
    }

    #[test]
    fn the_csi_u_format_is_selectable() {
        let modes = PaneKeyModes {
            extended: ExtendedKeys::All,
            ..PaneKeyModes::default()
        };
        let options = PaneKeyOptions {
            extended_keys_format: ExtendedKeysFormat::CsiU,
            ..PaneKeyOptions::default()
        };
        let code = parse_key_name("C-Enter").unwrap();
        assert_eq!(
            encode(PaneKey::from_name(code), modes, options).bytes,
            b"\x1b[13;5u"
        );
    }

    #[test]
    fn backtab_drops_its_modifiers_outside_extended_mode() {
        assert_eq!(bytes("BTab"), b"\x1b[Z");
        assert_eq!(bytes("C-BTab"), b"\x1b[Z");
        assert_eq!(bytes("S-Tab"), b"\t");
    }

    #[test]
    fn backspace_follows_the_backspace_option() {
        assert_eq!(bytes("BSpace"), b"\x7f");
        assert_eq!(bytes("M-BSpace"), b"\x1b\x7f");
        let options = PaneKeyOptions {
            backspace: parse_key_name("C-h").unwrap(),
            ..PaneKeyOptions::default()
        };
        let code = parse_key_name("BSpace").unwrap();
        assert_eq!(
            encode(PaneKey::from_name(code), PaneKeyModes::default(), options).bytes,
            b"\x08"
        );
    }

    #[test]
    fn meta_is_a_prefix_only_where_no_entry_spells_it_out() {
        assert_eq!(bytes("M-a"), b"\x1ba");
        assert_eq!(bytes("M-Space"), b"\x1b ");
        // `M-Up` has a built entry, so it must not also collect an `ESC`.
        assert_eq!(bytes("M-Up"), b"\x1b[1;3A");
        assert_eq!(bytes("M-F1"), b"\x1b[1;3P");
        // A key typed as `ESC` + `CSI A` carries no implied meta, so it does.
        let code = parse_key_name("M-Up").unwrap();
        let typed = PaneKey {
            code,
            cursor: false,
            keypad: false,
            implied_meta: false,
        };
        assert_eq!(
            encode(typed, PaneKeyModes::default(), PaneKeyOptions::default()).bytes,
            b"\x1b\x1b[A"
        );
    }

    #[test]
    fn plain_printable_and_control_keys_go_as_themselves() {
        assert_eq!(bytes("Tab"), b"\t");
        assert_eq!(bytes("Enter"), b"\r");
        assert_eq!(bytes("Escape"), b"\x1b");
        assert_eq!(bytes("Space"), b" ");
        assert_eq!(bytes("a"), b"a");
    }
}

//! tmux's `vis(3)` escaping, and the naming rule built on top of it.
//!
//! A pane title and a hyperlink URI are both *stored* escaped rather than raw:
//! `screen_set_title` and `hyperlinks_put` run their argument through
//! `utf8_stravis` before keeping it. That is observable, because `#{pane_title}`
//! and a `capture-pane -e` read back what was stored — a control byte in either
//! comes back as a three-digit octal escape, and a backslash comes back
//! doubled.
//!
//! Titles and paths go through one more step, `clean_name`, which refuses the
//! name outright unless every byte is either part of a valid UTF-8 character or
//! printable ASCII.

/// `VIS_TAB` and `VIS_NL`: whether tab and newline are escaped or passed
/// through. They are the only flags that differ between tmux's two callers —
/// both pass `VIS_OCTAL|VIS_CSTYLE` and neither passes `VIS_ALL`, `VIS_GLOB`,
/// `VIS_SP`, `VIS_SAFE`, `VIS_DQ` or `VIS_NOSLASH`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Whitespace {
    tab: bool,
    newline: bool,
}

/// `VIS_OCTAL|VIS_CSTYLE`, which is what `hyperlinks_put` escapes a URI with.
pub const KEEP_WHITESPACE: Whitespace = Whitespace {
    tab: false,
    newline: false,
};

/// `VIS_OCTAL|VIS_CSTYLE|VIS_TAB|VIS_NL`, which is what `clean_name` escapes a
/// title or a path with.
const ESCAPE_WHITESPACE: Whitespace = Whitespace {
    tab: true,
    newline: true,
};

/// tmux's `utf8_strvis`.
///
/// A complete, valid UTF-8 character is copied through untouched and everything
/// else is escaped one byte at a time. A lead byte that is not followed through
/// is *not* taken to consume the length it announced: only the lead is escaped,
/// and the bytes after it are read again from the start.
pub fn escape(bytes: &[u8], whitespace: Whitespace) -> String {
    let mut out = String::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match utf8_character(&bytes[index..]) {
            Some(size) => {
                let character = &bytes[index..index + size];
                out.push_str(std::str::from_utf8(character).expect("checked above"));
                index += size;
            }
            None => {
                push_escaped(&mut out, bytes[index], whitespace);
                index += 1;
            }
        }
    }
    out
}

/// tmux's `clean_name` with `untrusted` set, behind `screen_set_title` and
/// `screen_set_path`. `None` is tmux refusing the name, which leaves whatever
/// was set before it in place.
pub fn clean_name(bytes: &[u8]) -> Option<String> {
    if !is_valid(bytes) {
        return None;
    }
    // A stored `#(` would run a command every time the name is expanded as a
    // format, so the `#` is defused before the name is kept.
    let mut copy = bytes.to_vec();
    for index in 1..copy.len() {
        if copy[index] == b'(' && copy[index - 1] == b'#' {
            copy[index - 1] = b'_';
        }
    }
    Some(escape(&copy, ESCAPE_WHITESPACE))
}

/// tmux's `utf8_isvalid`, which is stricter than the name suggests: a byte that
/// is not part of a valid UTF-8 character has to be *printable* ASCII, so a
/// lone control byte fails this as surely as a malformed sequence does.
fn is_valid(bytes: &[u8]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        match utf8_character(&bytes[index..]) {
            Some(size) => index += size,
            None if (0x20..=0x7e).contains(&bytes[index]) => index += 1,
            None => return false,
        }
    }
    true
}

/// The length of the complete, valid UTF-8 character `bytes` opens with.
///
/// `utf8_open` takes the length from the lead byte alone and `utf8_append` then
/// requires the announced number of continuation bytes to arrive and decode, so
/// a truncated or malformed sequence is not a character at all.
fn utf8_character(bytes: &[u8]) -> Option<usize> {
    let size = match bytes.first()? {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };
    let candidate = bytes.get(..size)?;
    std::str::from_utf8(candidate).ok().map(|_| size)
}

/// One byte through `vis`, for the flags both callers share.
fn push_escaped(out: &mut String, byte: u8, whitespace: Whitespace) {
    // `isvisible` with neither VIS_ALL, VIS_GLOB, VIS_SP nor VIS_SAFE set:
    // printable ASCII, space always, and tab and newline unless asked for.
    let visible = byte.is_ascii_graphic()
        || byte == b' '
        || (byte == b'\t' && !whitespace.tab)
        || (byte == b'\n' && !whitespace.newline);
    if visible {
        // VIS_NOSLASH is not set, so a backslash arrives doubled.
        if byte == b'\\' {
            out.push('\\');
        }
        out.push(char::from(byte));
        return;
    }
    // VIS_CSTYLE spells the ones C spells. Space is on this list in `vis` too,
    // but only VIS_SP could send it here and neither caller sets that.
    let cstyle = match byte {
        0x07 => Some('a'),
        0x08 => Some('b'),
        b'\t' => Some('t'),
        b'\n' => Some('n'),
        0x0b => Some('v'),
        0x0c => Some('f'),
        b'\r' => Some('r'),
        _ => None,
    };
    if let Some(letter) = cstyle {
        out.push('\\');
        out.push(letter);
        return;
    }
    // VIS_OCTAL takes the rest, the high bytes included.
    out.push('\\');
    for shift in [6, 3, 0] {
        out.push(char::from(b'0' + ((byte >> shift) & 0o7)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_byte_is_escaped_in_octal_and_a_backslash_is_doubled() {
        assert_eq!(escape(b"http\x7fx", KEEP_WHITESPACE), r"http\177x");
        assert_eq!(escape(br"a\b", KEEP_WHITESPACE), r"a\\b");
        assert_eq!(escape(b"a\x01b", KEEP_WHITESPACE), r"a\001b");
    }

    #[test]
    fn a_byte_that_is_not_valid_utf8_is_escaped_rather_than_dropped() {
        assert_eq!(escape(b"http\xffx", KEEP_WHITESPACE), r"http\377x");
    }

    #[test]
    fn a_valid_character_passes_through_and_a_truncated_lead_does_not() {
        assert_eq!(escape("a€b".as_bytes(), KEEP_WHITESPACE), "a€b");
        // The `A` is read again rather than swallowed as the lead's third byte.
        assert_eq!(escape(b"\xe0A", KEEP_WHITESPACE), r"\340A");
    }

    #[test]
    fn tab_and_newline_follow_the_flags() {
        assert_eq!(escape(b"a\tb\nc", KEEP_WHITESPACE), "a\tb\nc");
        assert_eq!(escape(b"a\tb\nc", ESCAPE_WHITESPACE), r"a\tb\nc");
    }

    #[test]
    fn a_name_needs_valid_utf8_and_printable_ascii_throughout() {
        assert_eq!(clean_name(b"good").as_deref(), Some("good"));
        assert_eq!(clean_name("a€b".as_bytes()).as_deref(), Some("a€b"));
        // A control byte fails `utf8_isvalid` as surely as a malformed
        // sequence does, and the name is refused rather than escaped.
        assert_eq!(clean_name(b"ba\x7fd"), None);
        assert_eq!(clean_name(b"ba\x80d"), None);
        assert_eq!(clean_name(b"ba\td"), None);
    }

    #[test]
    fn a_name_defuses_a_format_command_and_doubles_a_backslash() {
        assert_eq!(clean_name(b"a#(whoami)b").as_deref(), Some("a_(whoami)b"));
        assert_eq!(clean_name(br"a\\b").as_deref(), Some(r"a\\\\b"));
        // Only a `#` that opens one is defused.
        assert_eq!(clean_name(b"a#b").as_deref(), Some("a#b"));
        assert_eq!(clean_name(b"a##(b").as_deref(), Some("a#_(b"));
    }
}

//! Answers the outer terminal sends back on the client's input stream.
//!
//! When the server asks an attached terminal a question — OSC 4 for a palette
//! colour, OSC 52 for the clipboard — the answer arrives on the client's tty,
//! interleaved with the user's keystrokes. [`AnswerScanner`] takes those reads
//! and intercepts the answers, mirroring tmux's `tty_keys_palette` and
//! `tty_keys_clipboard`; every byte that is not an answer passes through
//! untouched, in order.
//!
//! When to scan is the caller's decision: an OSC 4 or OSC 52 reply is only
//! recognisably the terminal's while the server is owed one, so the scanner
//! should be fed only inside that window.

use std::borrow::Cow;

use crate::observer::{base64_decode_strict, parse_packed_colour};

/// The most an unfinished answer may hold before it is given back as ordinary
/// input. A terminal that starts an answer and never finishes it must not
/// swallow what the user types after it.
const ANSWER_LIMIT: usize = 512;

/// An answer the terminal sent, parsed past its transport: the clipboard data
/// is decoded and the colour is packed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalAnswer {
    Palette {
        index: u8,
        /// The colour packed as `0xrrggbb`.
        colour: u32,
    },
    Clipboard {
        /// The selection letter the terminal named, if it named one.
        selection: Option<u8>,
        data: Vec<u8>,
    },
    /// `CSI ? … c`, the primary device attributes: what the terminal says it
    /// can do, as the numbers `tty_keys_device_attributes` reads.
    DeviceAttributes(Vec<u32>),
    /// `CSI > … c`, the secondary device attributes: which terminal it is.
    SecondaryDeviceAttributes(Vec<u32>),
    /// `DCS > | … ST`, XTVERSION: the name and version the terminal gives
    /// itself.
    TerminalVersion(String),
    /// An `OSC 10`/`OSC 11` answer to the server's own colour question, packed
    /// as `0xrrggbb`.
    TerminalColour { number: u32, colour: u32 },
}

/// How far parsing got with the head of the buffer.
enum Progress {
    /// Not the start of an answer.
    None,
    /// The start of one, but the terminator has not arrived yet.
    Partial,
    /// A whole answer, and how many bytes of the input it took.
    Complete(TerminalAnswer, usize),
}

/// Picks the server's answers out of a client's tty reads, holding an
/// unfinished one over a read boundary until its terminator arrives.
#[derive(Default)]
pub struct AnswerScanner {
    pending: Vec<u8>,
    /// Which of `OSC 10` and `OSC 11` the server is still owed an answer to,
    /// and so takes for itself rather than leaving to a pane that asked the
    /// same question — tmux's `TTY_WAITFG` and `TTY_WAITBG`.
    expect_foreground: bool,
    expect_background: bool,
}

impl AnswerScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take the terminal's answers to the server's own `OSC 10`/`OSC 11`
    /// questions, until they have arrived.
    pub fn expect_colours(&mut self, foreground: bool, background: bool) {
        self.expect_foreground = foreground;
        self.expect_background = background;
    }

    /// Whether an unfinished answer is held from an earlier read — bytes that
    /// must not reach a pane until the answer resolves one way or the other.
    pub fn is_holding(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Scan one tty read: the bytes that are not answers, in order, and the
    /// answers found. An answer the read cut off is held for the next call,
    /// up to a limit past which it turns back into ordinary input.
    pub fn feed(&mut self, data: &[u8]) -> (Vec<u8>, Vec<TerminalAnswer>) {
        // Holding an unfinished answer is the rare case; with nothing held the
        // read is scanned where it lies rather than copied to be joined to an
        // empty buffer first.
        let buffered = if self.pending.is_empty() {
            Cow::Borrowed(data)
        } else {
            let mut joined = std::mem::take(&mut self.pending);
            joined.extend_from_slice(data);
            Cow::Owned(joined)
        };
        let mut passthrough = Vec::with_capacity(buffered.len());
        let mut answers = Vec::new();
        let mut index = 0;
        // Everything since the last answer, copied out in one run when an
        // answer ends it rather than a byte at a time as it is walked.
        let mut run = 0;
        let expect = (self.expect_foreground, self.expect_background);
        while index < buffered.len() {
            match parse_answer(&buffered[index..], expect) {
                Progress::Complete(answer, consumed) => {
                    passthrough.extend_from_slice(&buffered[run..index]);
                    index += consumed;
                    run = index;
                    answers.push(answer);
                }
                Progress::Partial if buffered.len() - index < ANSWER_LIMIT => {
                    passthrough.extend_from_slice(&buffered[run..index]);
                    self.pending = buffered[index..].to_vec();
                    return (passthrough, answers);
                }
                Progress::None | Progress::Partial => index += 1,
            }
        }
        passthrough.extend_from_slice(&buffered[run..]);
        (passthrough, answers)
    }
}

/// Recognize an answer at the head of `data`: an OSC 4 palette colour, an
/// OSC 52 clipboard payload, or one of the capability replies
/// `tty_send_requests` asks for.
///
/// These are the answers the server routes itself; every other byte belongs to
/// whoever is reading the client's input.
fn parse_answer(data: &[u8], expect_colours: (bool, bool)) -> Progress {
    let expects = |number: u32| match number {
        10 => expect_colours.0,
        _ => expect_colours.1,
    };
    match parse_capability_answer(data) {
        Progress::None => {}
        progress => return progress,
    }
    let Some(rest) = data.strip_prefix(b"\x1b]") else {
        return Progress::None;
    };
    let colour_number = [10u32, 11]
        .into_iter()
        .find(|number| expects(*number) && rest.starts_with(format!("{number};").as_bytes()));
    let (palette, body) = if let Some(number) = colour_number {
        (false, &rest[number.to_string().len() + 1..])
    } else if let Some(body) = rest.strip_prefix(b"4;") {
        (true, body)
    } else if let Some(body) = rest.strip_prefix(b"52;") {
        (false, body)
    } else {
        // A prefix of any introducer is still worth waiting on, once the
        // introducer itself is not all there is.
        return if !rest.is_empty()
            && (b"4;".starts_with(rest)
                || b"52;".starts_with(rest)
                || (expects(10) && b"10;".starts_with(rest))
                || (expects(11) && b"11;".starts_with(rest)))
        {
            Progress::Partial
        } else {
            Progress::None
        };
    };
    let Some((end, terminator)) = body
        .iter()
        .position(|byte| *byte == 0x07)
        .map(|end| (end, 1))
        .or_else(|| {
            body.windows(2)
                .position(|window| window == b"\x1b\\")
                .map(|end| (end, 2))
        })
    else {
        return Progress::Partial;
    };
    let consumed = data.len() - body.len() + end + terminator;
    let payload = &body[..end];
    if let Some(number) = colour_number {
        let colour = std::str::from_utf8(payload)
            .ok()
            .and_then(parse_packed_colour)
            .unwrap_or(0);
        return Progress::Complete(TerminalAnswer::TerminalColour { number, colour }, consumed);
    }
    if palette {
        let Some((index, colour)) = std::str::from_utf8(payload)
            .ok()
            .and_then(|text| text.split_once(';'))
        else {
            return Progress::Complete(
                TerminalAnswer::Palette {
                    index: 0,
                    colour: 0,
                },
                consumed,
            );
        };
        let parsed = index.parse::<u8>().ok().zip(parse_packed_colour(colour));
        return match parsed {
            // An answer that parses to nothing still belongs to the server: it
            // answered a question no pane should see.
            None => Progress::Complete(
                TerminalAnswer::Palette {
                    index: u8::MAX,
                    colour: 0,
                },
                consumed,
            ),
            Some((index, colour)) => {
                Progress::Complete(TerminalAnswer::Palette { index, colour }, consumed)
            }
        };
    }
    // `\033]52;<selection>;<base64>`. tmux takes the selection only when it is
    // a single letter before the second `;`.
    let (selection, encoded) = match payload.iter().position(|byte| *byte == b';') {
        Some(split) => (
            (split == 1).then(|| payload[0]),
            &payload[split.saturating_add(1)..],
        ),
        None => (None, &[][..]),
    };
    Progress::Complete(
        TerminalAnswer::Clipboard {
            selection,
            data: std::str::from_utf8(encoded)
                .ok()
                .and_then(base64_decode_strict)
                .unwrap_or_default(),
        },
        consumed,
    )
}

/// Recognize a device-attributes or XTVERSION reply at the head of `data`.
fn parse_capability_answer(data: &[u8]) -> Progress {
    if let Some(rest) = data.strip_prefix(b"\x1b[") {
        let Some((marker, body)) = rest.split_first() else {
            return Progress::Partial;
        };
        let secondary = match marker {
            b'?' => false,
            b'>' => true,
            _ => return Progress::None,
        };
        let Some(end) = body.iter().position(|byte| byte.is_ascii_alphabetic()) else {
            return Progress::Partial;
        };
        if body[end] != b'c' {
            return Progress::None;
        }
        let parameters = numeric_parameters(&body[..end]);
        let consumed = data.len() - body.len() + end + 1;
        let answer = if secondary {
            TerminalAnswer::SecondaryDeviceAttributes(parameters)
        } else {
            TerminalAnswer::DeviceAttributes(parameters)
        };
        return Progress::Complete(answer, consumed);
    }
    if let Some(body) = data.strip_prefix(b"\x1bP>|") {
        let Some(end) = body.windows(2).position(|window| window == b"\x1b\\") else {
            return Progress::Partial;
        };
        let text = String::from_utf8_lossy(&body[..end]).into_owned();
        let consumed = data.len() - body.len() + end + 2;
        return Progress::Complete(TerminalAnswer::TerminalVersion(text), consumed);
    }
    // A prefix of an introducer is only worth waiting on once it is more than
    // the escape itself: a lone `ESC` is the user's key far more often than the
    // head of a reply, and holding it would keep it from whoever is reading.
    for introducer in [b"\x1b[?".as_slice(), b"\x1b[>".as_slice(), b"\x1bP>|".as_slice()] {
        if data.len() > 1 && introducer.starts_with(data) {
            return Progress::Partial;
        }
    }
    Progress::None
}

/// The `;`-separated numbers of a device-attributes reply, with an unreadable
/// field counted as zero the way `strtoul` leaves it.
fn numeric_parameters(body: &[u8]) -> Vec<u32> {
    String::from_utf8_lossy(body)
        .split(';')
        .map(|field| field.parse::<u32>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(data: &[u8]) -> (Vec<u8>, Vec<TerminalAnswer>) {
        AnswerScanner::new().feed(data)
    }

    #[test]
    fn a_palette_answer_is_intercepted_and_parsed() {
        let (kept, answers) = scan(b"\x1b]4;1;rgb:ff/80/00\x07");
        assert!(kept.is_empty());
        assert_eq!(
            answers,
            [TerminalAnswer::Palette {
                index: 1,
                colour: 0xff8000,
            }]
        );
    }

    #[test]
    fn a_clipboard_answer_decodes_its_base64_and_names_its_selection() {
        let (kept, answers) = scan(b"\x1b]52;c;aGk=\x1b\\");
        assert!(kept.is_empty());
        assert_eq!(
            answers,
            [TerminalAnswer::Clipboard {
                selection: Some(b'c'),
                data: b"hi".to_vec(),
            }]
        );
    }

    #[test]
    fn unpadded_base64_decodes_to_nothing_as_tmux_drops_it() {
        let (_, answers) = scan(b"\x1b]52;c;aGk\x07");
        assert_eq!(
            answers,
            [TerminalAnswer::Clipboard {
                selection: Some(b'c'),
                data: Vec::new(),
            }]
        );
    }

    #[test]
    fn keystrokes_around_an_answer_pass_through_in_order() {
        let (kept, answers) = scan(b"ab\x1b]52;c;aGk=\x07cd");
        assert_eq!(kept, b"abcd");
        assert_eq!(answers.len(), 1);
    }

    #[test]
    fn an_answer_cut_off_by_the_read_is_held_for_the_next_one() {
        let mut scanner = AnswerScanner::new();
        let (kept, answers) = scanner.feed(b"\x1b]4;1;rgb:ff/f");
        assert!(kept.is_empty() && answers.is_empty());
        assert!(scanner.is_holding());
        let (kept, answers) = scanner.feed(b"f/ff\x07x");
        assert_eq!(kept, b"x");
        assert_eq!(
            answers,
            [TerminalAnswer::Palette {
                index: 1,
                colour: 0xffffff,
            }]
        );
        assert!(!scanner.is_holding());
    }

    #[test]
    fn a_bare_introducer_prefix_is_held_rather_than_leaked() {
        let mut scanner = AnswerScanner::new();
        let (kept, answers) = scanner.feed(b"\x1b]5");
        assert!(kept.is_empty() && answers.is_empty());
        assert!(scanner.is_holding());
    }

    #[test]
    fn an_answer_that_never_terminates_turns_back_into_input() {
        let mut scanner = AnswerScanner::new();
        let unterminated = [b"\x1b]52;c;".as_slice(), &[b'A'; ANSWER_LIMIT]].concat();
        let (kept, answers) = scanner.feed(&unterminated);
        assert_eq!(kept, unterminated);
        assert!(answers.is_empty());
        assert!(!scanner.is_holding());
    }

    #[test]
    fn a_malformed_palette_answer_is_still_consumed_not_forwarded() {
        let (kept, answers) = scan(b"\x1b]4;bogus\x07");
        assert!(kept.is_empty());
        assert_eq!(
            answers,
            [TerminalAnswer::Palette {
                index: 0,
                colour: 0,
            }]
        );
    }

    #[test]
    fn an_unrelated_osc_passes_through_untouched() {
        let (kept, answers) = scan(b"\x1b]0;title\x07");
        assert_eq!(kept, b"\x1b]0;title\x07");
        assert!(answers.is_empty());
    }
}

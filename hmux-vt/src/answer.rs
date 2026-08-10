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
}

impl AnswerScanner {
    pub fn new() -> Self {
        Self::default()
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
        let mut buffered = std::mem::take(&mut self.pending);
        buffered.extend_from_slice(data);
        let mut passthrough = Vec::with_capacity(buffered.len());
        let mut answers = Vec::new();
        let mut index = 0;
        while index < buffered.len() {
            match parse_answer(&buffered[index..]) {
                Progress::Complete(answer, consumed) => {
                    index += consumed;
                    answers.push(answer);
                }
                Progress::Partial if buffered.len() - index < ANSWER_LIMIT => {
                    self.pending = buffered[index..].to_vec();
                    return (passthrough, answers);
                }
                Progress::None | Progress::Partial => {
                    passthrough.push(buffered[index]);
                    index += 1;
                }
            }
        }
        (passthrough, answers)
    }
}

/// Recognize an OSC 4 or OSC 52 answer at the head of `data`.
///
/// These are the only two terminal answers the server routes itself; every
/// other byte belongs to whoever is reading the client's input.
fn parse_answer(data: &[u8]) -> Progress {
    let Some(rest) = data.strip_prefix(b"\x1b]") else {
        return Progress::None;
    };
    let (palette, body) = if let Some(body) = rest.strip_prefix(b"4;") {
        (true, body)
    } else if let Some(body) = rest.strip_prefix(b"52;") {
        (false, body)
    } else {
        // A prefix of either introducer is still worth waiting on.
        return if b"4;".starts_with(rest) || b"52;".starts_with(rest) {
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

//! Running both backends over the same bytes and comparing what they hold.
//!
//! The seam makes this cheap: two implementations of one trait, fed the same
//! tokens, read back the same way. It is the check that keeps the in-house
//! engine honest while it is being built, and the way a disagreement is found
//! before the conformance suite has to find it in a pane.
//!
//! This is test-only and never shipped. Per the repository's
//! competing-implementation rule, the product carries one engine; two exist
//! here purely so they can be diffed.

#![cfg(test)]

use super::engine::backend::EngineScreen;
use super::ghostty::GhosttyScreen;
use super::input::{InputEncoder, MouseAction, MouseButton, MouseEvent};
use super::parser::Parser;
use super::screen::VtScreen;

/// What both backends say about a screen after the same input.
#[derive(Debug, Eq, PartialEq)]
struct Readback {
    plain: String,
    unwrapped: String,
    cursor: (u16, u16),
    cursor_visible: bool,
    scrollback_rows: usize,
    /// The snapshot reduced to per-row text, which is what a consumer of
    /// `capture-pane` or copy mode actually reads.
    rows: Vec<String>,
}

fn read_back(screen: &dyn VtScreen) -> Readback {
    let grid = screen.grid_snapshot().expect("snapshot");
    Readback {
        plain: screen.dump_plain().expect("plain"),
        unwrapped: screen.dump_plain_unwrapped().expect("unwrapped"),
        cursor: screen.cursor_position().expect("cursor"),
        cursor_visible: screen.cursor_visible().expect("cursor mode"),
        scrollback_rows: screen.scrollback_rows().expect("history"),
        rows: grid
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.text.as_str())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect(),
    }
}

/// Feed one stream to both backends and read both back.
///
/// One parser drives both, across all the chunks: the point of a split-read
/// case is that the *parser* holds state over the boundary, so tokenizing each
/// chunk on its own would test nothing.
fn compare(cols: u16, rows: u16, chunks: &[&[u8]]) -> (Readback, Readback) {
    let mut ours = EngineScreen::new(cols, rows);
    let mut theirs = GhosttyScreen::new(cols, rows).expect("ghostty screen");
    let mut parser = Parser::default();
    for chunk in chunks {
        parser.parse(chunk, |token| {
            ours.apply(&token);
            VtScreen::apply(&mut theirs, &token);
        });
    }
    (read_back(&ours), read_back(&theirs))
}

/// Assert both backends agree on the parts of the readback that are not
/// already known to diverge.
fn assert_same(name: &str, cols: u16, rows: u16, chunks: &[&[u8]]) {
    let (ours, theirs) = compare(cols, rows, chunks);
    assert_eq!(ours.plain, theirs.plain, "{name}: plain text");
    assert_eq!(ours.cursor, theirs.cursor, "{name}: cursor");
    assert_eq!(
        ours.cursor_visible, theirs.cursor_visible,
        "{name}: cursor visibility"
    );
    assert_eq!(
        ours.scrollback_rows, theirs.scrollback_rows,
        "{name}: scrollback rows"
    );
}

#[test]
fn plain_output_agrees() {
    assert_same("text", 20, 4, &[b"hello\r\nworld"]);
    assert_same("overwrite", 20, 4, &[b"AAAAA\rBB"]);
    assert_same("many rows", 20, 3, &[b"a\r\nb\r\nc\r\nd\r\ne\r\nf"]);
}

#[test]
fn wrapping_agrees() {
    assert_same("soft wrap", 6, 3, &[b"abcdefghij"]);
    assert_same("wrap off", 6, 3, &[b"\x1b[?7labcdefghij"]);
}

#[test]
fn cursor_movement_agrees() {
    assert_same("cup", 20, 5, &[b"\x1b[3;5Hx"]);
    assert_same("relative", 20, 5, &[b"abc\x1b[2A\x1b[3Cx"]);
    assert_same("column", 20, 5, &[b"abcdef\x1b[3Gx"]);
    assert_same("row", 20, 5, &[b"\x1b[4dx"]);
    assert_same("home", 20, 5, &[b"abc\r\ndef\x1b[Hx"]);
}

#[test]
fn erasing_agrees() {
    assert_same("end of line", 20, 3, &[b"abcdef\x1b[1;4H\x1b[K"]);
    assert_same("start of line", 20, 3, &[b"abcdef\x1b[1;4H\x1b[1K"]);
    assert_same("whole line", 20, 3, &[b"abcdef\x1b[2K"]);
    assert_same("end of screen", 20, 3, &[b"a\r\nb\r\nc\x1b[2;1H\x1b[J"]);
    assert_same("whole screen", 20, 3, &[b"a\r\nb\r\nc\x1b[2J"]);
}

#[test]
fn scrolling_and_regions_agree() {
    assert_same("scroll up", 20, 4, &[b"a\r\nb\r\nc\r\nd\x1b[2S"]);
    assert_same("scroll down", 20, 4, &[b"a\r\nb\r\nc\r\nd\x1b[1T"]);
    assert_same("reverse index", 20, 4, &[b"a\r\nb\x1b[H\x1bM"]);
}

/// The mouse encoders have to agree byte for byte: the bytes go straight to
/// the program in the pane, and a difference there is a difference the program
/// sees.
#[test]
fn mouse_reports_agree() {
    let modes: [&[u8]; 6] = [
        b"",
        b"\x1b[?1000h",
        b"\x1b[?1000h\x1b[?1006h",
        b"\x1b[?1002h\x1b[?1006h",
        b"\x1b[?1003h\x1b[?1006h",
        b"\x1b[?1000h\x1b[?1005h",
    ];
    let buttons = [
        Some(MouseButton::Left),
        Some(MouseButton::Middle),
        Some(MouseButton::Right),
        Some(MouseButton::WheelUp),
        Some(MouseButton::WheelDown),
        None,
    ];
    for setup in modes {
        let mut ours = EngineScreen::new(80, 24);
        let mut theirs = GhosttyScreen::new(80, 24).expect("ghostty screen");
        let mut parser = Parser::default();
        parser.parse(setup, |token| {
            ours.apply(&token);
            VtScreen::apply(&mut theirs, &token);
        });
        for action in [
            MouseAction::Press,
            MouseAction::Release,
            MouseAction::Motion,
        ] {
            for button in buttons {
                for held in [false, true] {
                    for (shift, control, alt) in [
                        (false, false, false),
                        (true, false, false),
                        (false, true, true),
                    ] {
                        let event = MouseEvent {
                            action,
                            button,
                            shift,
                            control,
                            alt,
                            column: 2,
                            row: 3,
                            any_button_pressed: held,
                        };
                        assert_eq!(
                            ours.encode_mouse(event).expect("ours"),
                            theirs.encode_mouse(event).expect("theirs"),
                            "setup {:?} event {event:?}",
                            String::from_utf8_lossy(setup)
                        );
                    }
                }
            }
        }
    }
}

// ---- where the two backends legitimately disagree --------------------------
//
// The cases below are the conformance ceiling the in-house engine exists to
// lift. Each records what tmux 3.7b itself answers, checked against the pinned
// oracle, so the direction of the difference is stated rather than inferred.

/// tmux parks the cursor one past the last column while a wrap is pending;
/// libghostty-vt keeps it on the last column and tracks the wrap separately.
/// Checked against the oracle: `printf 'abcde'` in a five-column tmux pane
/// reports `#{cursor_x}` of 5.
#[test]
fn the_pending_wrap_column_is_where_the_engine_diverges() {
    let (ours, theirs) = compare(5, 3, &[b"abcde"]);
    assert_eq!(ours.cursor.0, 5, "the in-house engine answers as tmux does");
    assert_eq!(theirs.cursor.0, 4, "libghostty-vt stops at the last column");
    assert_eq!(ours.plain, theirs.plain, "the text itself still agrees");
}

/// Scrolling a region whose top is not the top of the screen still feeds the
/// history in tmux; libghostty-vt drops the row. Checked against the oracle:
/// the stream below leaves tmux with `#{history_size}` of 1 and the scrolled
/// row above the viewport.
#[test]
fn a_partial_region_scroll_is_where_the_history_diverges() {
    let stream = b"a\r\nb\r\nc\r\nd\r\ne\x1b[2;4r\x1b[4;1H\n";
    let (ours, theirs) = compare(20, 5, &[stream]);
    assert_eq!(
        ours.plain, "b\na\nc\nd\n\ne",
        "the in-house engine keeps the scrolled row, as tmux does"
    );
    assert_eq!(ours.scrollback_rows, 1);
    assert_eq!(theirs.plain, "a\nc\nd\n\ne", "libghostty-vt drops it");
    assert_eq!(theirs.scrollback_rows, 0);
}

#[test]
fn insert_and_delete_agree() {
    assert_same("insert chars", 20, 3, &[b"abcdef\x1b[1;3H\x1b[2@"]);
    assert_same("delete chars", 20, 3, &[b"abcdef\x1b[1;3H\x1b[2P"]);
    assert_same("insert lines", 20, 4, &[b"a\r\nb\r\nc\x1b[2;1H\x1b[L"]);
    assert_same("delete lines", 20, 4, &[b"a\r\nb\r\nc\x1b[1;1H\x1b[M"]);
    assert_same("erase chars", 20, 3, &[b"abcdef\x1b[1;2H\x1b[3X"]);
}

#[test]
fn unicode_agrees() {
    assert_same("wide", 10, 3, &["a界b".as_bytes()]);
    assert_same("wide at margin", 5, 3, &["abcd界".as_bytes()]);
    assert_same("combining", 10, 3, &["e\u{301}x".as_bytes()]);
    assert_same("malformed", 10, 3, &[b"a\xc3\x28b"]);
}

#[test]
fn split_reads_do_not_change_the_answer() {
    assert_same(
        "split sequence",
        10,
        3,
        &[b"abc\x1b[1", b";1Hx\x1b[3", b"1mY"],
    );
    assert_same("split utf8", 10, 3, &["界".as_bytes(), "x".as_bytes()]);
}

#[test]
fn the_alternate_screen_agrees() {
    assert_same(
        "enter and leave",
        20,
        3,
        &[b"primary\x1b[?1049halt\x1b[?1049l"],
    );
}

#[test]
fn cursor_visibility_agrees() {
    assert_same("hidden", 10, 2, &[b"\x1b[?25l"]);
    assert_same("shown again", 10, 2, &[b"\x1b[?25l\x1b[?25h"]);
}

#[test]
fn a_long_stream_agrees() {
    // Enough output to push rows through the history and exercise the
    // collection path on both sides.
    let mut stream = Vec::new();
    for n in 0..200 {
        stream.extend_from_slice(format!("line {n:03}\r\n").as_bytes());
    }
    assert_same("scrollback", 20, 5, &[&stream]);
}

/// The tab origin is the case the plan names as ghostty-blocked: tmux records
/// a tab-created run as one cell so `capture-pane -e` can tell it from typed
/// spaces, and libghostty-vt has no such distinction. Both backends still have
/// to agree on the *text*; the difference is only in what the cells carry.
#[test]
fn a_tab_agrees_on_text_even_where_the_cells_differ() {
    assert_same("tab", 20, 2, &[b"\tX"]);
    assert_same("tab after text", 20, 2, &[b"ab\tX"]);
}

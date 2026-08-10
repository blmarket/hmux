//! The emulator itself: bytes in, ordered events out.
//!
//! [`Terminal`] owns the whole chain a pane's output flows through — the
//! tokenizer, the observer that applies the pane's options, the screen the
//! surviving tokens land on, and the scroll classification the compositor's
//! repaint choice needs. [`Terminal::process`] takes one chunk of the byte
//! stream and returns everything the server has to act on as one ordered
//! [`TerminalEvent`] list.
//!
//! Every event is self-contained: state it reports — a cursor position, a
//! mode word — is captured from the screen at the event's own point in the
//! stream. Applying the chunk and then walking the events therefore sees
//! exactly what handling each event mid-stream would have seen, which is what
//! lets the interface be a plain returned `Vec` instead of a callback into
//! the application loop. The only ordering the caller owes the events is the
//! `Vec`'s own.

use crate::observer::{decrqss_reply, CursorShape, Event, Observer, OscState, OutputPolicy};
use crate::parser::{tokenize, StringEnd, Token};
use crate::screen::{mode, PaneScreen, ScreenOptions};
use crate::scroll::ScrollRedraw;
use crate::ClipboardEvent;

/// Something in the processed byte stream the server has to act on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalEvent {
    /// A `BEL` reached the screen.
    Bell,
    /// `ESC k … ST`: the screen-family window rename control.
    Rename(String),
    /// The pane switched screens (DECSET 47 / 1047 / 1049).
    AlternateScreen(bool),
    /// DECSET 1049 is about to switch, and this is the cursor to remember —
    /// read before the switch, because the switch is what moves it.
    SaveAlternateCursor { x: u16, y: u16 },
    /// Bytes to write back to the pane's own input.
    Reply(Vec<u8>),
    /// Bytes to forward to the client's terminal, whose answer comes back to
    /// the pane.
    ForwardQuery(&'static [u8]),
    /// OSC 4 asked for a palette entry neither the pane nor `pane-colours`
    /// has. tmux asks the attached terminal instead and answers the pane with
    /// what comes back, so the index and the query's terminator are carried
    /// until the server can route it.
    PaletteQuery { index: u8, end: StringEnd },
    /// An OSC 52 clipboard set or query.
    Clipboard(ClipboardEvent),
    /// A `DCS tmux;` payload, already stripped of its prefix and terminator.
    Passthrough(Vec<u8>),
    /// DSR ?996: the pane asked which theme it is running under.
    ThemeQuery,
    /// Something in this chunk scrolled enough of the pane that the compositor
    /// should repaint the whole thing rather than the moved rows. Emitted at
    /// most once per [`Terminal::process`] call, after the other events.
    LargeScroll,
}

/// One pane's terminal emulation: tokenizer, per-pane observer state, and the
/// screen, driven together so their interleaving is nobody else's concern.
pub struct Terminal {
    observer: Observer,
    screen: PaneScreen,
    scroll: ScrollRedraw,
}

impl Terminal {
    pub fn new(cols: u16, rows: u16) -> Terminal {
        Terminal {
            observer: Observer::default(),
            screen: PaneScreen::new(cols, rows),
            scroll: ScrollRedraw::new(rows),
        }
    }

    /// The screen, for everything reading it back out: grids, dumps, cursor,
    /// modes, images.
    pub fn screen(&self) -> &PaneScreen {
        &self.screen
    }

    /// Apply the pane options the screen consults; see [`ScreenOptions`].
    pub fn set_screen_options(&mut self, options: ScreenOptions) {
        self.screen.set_options(options);
    }

    /// Apply a pane's history limit to the retained primary-screen scrollback.
    pub fn set_history_limit(&mut self, limit: usize) {
        self.screen.set_history_limit(limit);
    }

    /// `resize-pane -T`: drop the rows below the cursor and pull the same
    /// number of rows out of the history in their place; see
    /// [`PaneScreen::trim_history_below_cursor`].
    pub fn trim_history_below_cursor(&mut self) {
        self.screen.trim_history_below_cursor();
    }

    /// Resize the screen, keeping the scroll classification in step.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.screen.resize(cols, rows);
        self.scroll.resize(rows);
    }

    /// The scroll region as `#{scroll_region_upper}`/`#{lower}` report it: the
    /// DECSTBM region when one is set, else the whole screen.
    pub fn scroll_region(&self) -> (u16, u16) {
        self.scroll.region()
    }

    /// The bytes of a sequence the tokenizer has not finished, which
    /// `capture-pane -P` returns.
    pub fn pending(&self) -> &[u8] {
        self.observer.pending()
    }

    /// The pane state OSC sequences have set — colours, path, progress — as
    /// the formats report it.
    pub fn osc_state(&self) -> OscState {
        self.observer.osc_state()
    }

    /// Whether the tokenizer is waiting for a string terminator, which is when
    /// tmux's ground timer runs.
    pub fn awaiting_terminator(&self) -> bool {
        self.observer.awaiting_terminator()
    }

    /// Process one chunk of the pane's byte stream: apply what the screen
    /// should show, and return the events the server has to act on, in stream
    /// order and self-contained.
    pub fn process(&mut self, input: &[u8], policy: &OutputPolicy) -> Vec<TerminalEvent> {
        let observed = self.observer.feed(input, policy);
        let tokens = &observed.screen[..];
        let mut events = Vec::with_capacity(observed.events.len());
        let mut large_scroll = false;
        let mut applied = 0usize;
        for (split_at, event) in observed.events {
            while applied < split_at {
                large_scroll |= self.apply(&tokens[applied]);
                applied += 1;
            }
            let resolved = self.resolve(event, policy);
            events.push(resolved);
        }
        for token in &tokens[applied..] {
            large_scroll |= self.apply(token);
        }
        if large_scroll {
            events.push(TerminalEvent::LargeScroll);
        }
        events
    }

    /// Reset the terminal as `RIS` does, without any bytes from the pane.
    pub fn reset(&mut self) {
        self.apply_sequence(b"\x1bc");
    }

    /// Erase the scrollback as `CSI 3 J` does — the emulator's own operation,
    /// so the grid is not reconstructed outside it.
    pub fn erase_scrollback(&mut self) {
        self.apply_sequence(b"\x1b[3J");
    }

    /// Give up on a string sequence whose terminator never arrived, as tmux's
    /// ground timer does, and report whether there was one.
    ///
    /// tmux's `input_ground_timer_callback` reaches `input_reset(ictx, 0)`,
    /// which is more than the tokenizer's half: it also returns the pending
    /// cell and the charset designations to their defaults. Those are the
    /// screen's, so they are reset by synthesizing the sequences that say it.
    /// What `input_reset` does to the DECSC save is not reachable that way and
    /// is left alone; nothing the server reports exposes it.
    pub fn expire_ground(&mut self) -> bool {
        if !self.observer.expire() {
            return false;
        }
        self.apply_sequence(b"\x1b[m\x0f\x1b(B\x1b)B");
        true
    }

    /// Apply one screen-bound token, reporting whether it scrolled enough of
    /// the pane for a whole repaint. The cursor is read *before* the token
    /// applies, because a scroll is only visible where the cursor already was.
    fn apply(&mut self, token: &Token) -> bool {
        let large = self.scroll.scan(token, self.screen.cursor_position().1);
        self.screen.apply(token);
        large
    }

    fn apply_sequence(&mut self, sequence: &[u8]) {
        for token in tokenize(sequence) {
            self.apply(&token);
        }
    }

    /// Turn an observed event into its public form, capturing the screen state
    /// it reports at this point in the stream. The queries whose whole answer
    /// is emulator state — DECRQSS, DECRQM — resolve to the reply bytes
    /// themselves; the policy supplies the `cursor-style` fallback their
    /// cursor answers share.
    fn resolve(&self, event: Event, policy: &OutputPolicy) -> TerminalEvent {
        match event {
            Event::Bell => TerminalEvent::Bell,
            Event::Rename(name) => TerminalEvent::Rename(name),
            Event::AlternateScreen(on) => TerminalEvent::AlternateScreen(on),
            Event::SaveAlternateCursor => {
                let (x, y) = self.screen.cursor_position();
                TerminalEvent::SaveAlternateCursor { x, y }
            }
            // DSR 6n, answered with the cursor as it stands here — which is
            // why the event is ordered inside the stream at all.
            Event::CursorPositionReport => {
                let (x, y) = self.screen.cursor_position();
                TerminalEvent::Reply(
                    format!("\x1b[{};{}R", y.saturating_add(1), x.saturating_add(1)).into_bytes(),
                )
            }
            Event::WindowSizeReport(operation) => {
                TerminalEvent::Reply(window_size_report(&self.screen, operation))
            }
            Event::DecPrivateModeReport(mode) => {
                let status = dec_mode_status(&self.screen, policy.cursor_style, mode);
                TerminalEvent::Reply(format!("\x1b[?{mode};{status}$y").into_bytes())
            }
            Event::DecModeReport(mode) => {
                let status = ansi_mode_status(self.screen.modes(), mode);
                TerminalEvent::Reply(format!("\x1b[{mode};{status}$y").into_bytes())
            }
            Event::StatusReport(request) => TerminalEvent::Reply(decrqss_reply(
                &request,
                CursorShape::from_parameter(self.screen.cursor_style()),
                self.screen.modes() & mode::CURSOR_BLINKING != 0,
                policy.cursor_style,
            )),
            Event::Reply(bytes) => TerminalEvent::Reply(bytes),
            Event::ForwardQuery(query) => TerminalEvent::ForwardQuery(query),
            Event::PaletteQuery { index, end } => TerminalEvent::PaletteQuery { index, end },
            Event::Clipboard(clipboard) => TerminalEvent::Clipboard(clipboard),
            Event::Passthrough(data) => TerminalEvent::Passthrough(data),
            Event::ThemeQuery => TerminalEvent::ThemeQuery,
        }
    }
}

/// tmux's XTWINOPS geometry reports — the only operations `CSI t` answers,
/// and the only ones the observer reports, so every reachable operation has
/// an arm. The pixel forms use the cell size the server pushed with the
/// screen options: the attached clients' geometry, or tmux's defaults while
/// no client has reported one.
fn window_size_report(screen: &PaneScreen, operation: u32) -> Vec<u8> {
    let dims = screen.grid_dims();
    let options = screen.options();
    let (cols, rows) = (u32::from(dims.cols), u32::from(dims.viewport_rows));
    let (report, height, width) = match operation {
        14 => (4, rows * options.ypixel, cols * options.xpixel),
        15 => (5, rows * options.ypixel, cols * options.xpixel),
        16 => (6, options.ypixel, options.xpixel),
        18 => (8, rows, cols),
        _ => (9, rows, cols),
    };
    format!("\x1b[{report};{height};{width}t").into_bytes()
}

/// tmux's DECRQM answer for a private mode: 1 set, 2 reset, 0 unrecognized,
/// or 4 permanently reset.
///
/// Everything it reads is the screen's at this point in the stream: the mode
/// word, the alternate-screen state the 47/1047/1049 aliases stand for, and
/// the DECSCUSR style. Mode 12 is the one answer that is not a mode-word read.
/// tmux reports the blink the pane asked for only once the pane has spoken —
/// a DECSCUSR that moved the cursor off its default style, or a DECSET/DECRST
/// 12 — and otherwise answers from the `cursor-style` option, whose blinking
/// choices are the odd parameters.
fn dec_mode_status(screen: &PaneScreen, cursor_style: u8, mode_number: u32) -> u8 {
    let modes = screen.modes();
    let reports = |bit: u32| Some(modes & bit != 0);
    let set = match mode_number {
        // DECCOLM is recognized by tmux but permanently reset: hmux has no
        // 132-column screen mode to enable either.
        3 => return 4,
        1 => reports(mode::KCURSOR),
        6 => reports(mode::ORIGIN),
        7 => reports(mode::WRAP),
        12 => Some(
            if CursorShape::from_parameter(screen.cursor_style()) != CursorShape::Default
                || modes & mode::CURSOR_BLINKING_SET != 0
            {
                modes & mode::CURSOR_BLINKING != 0
            } else {
                matches!(cursor_style, 1 | 3 | 5)
            },
        ),
        25 => reports(mode::CURSOR),
        47 | 1047 | 1049 => Some(screen.alternate_active()),
        1000 => reports(mode::MOUSE_STANDARD),
        1002 => reports(mode::MOUSE_BUTTON),
        1003 => reports(mode::MOUSE_ALL),
        1004 => reports(mode::FOCUSON),
        1005 => reports(mode::MOUSE_UTF8),
        1006 => reports(mode::MOUSE_SGR),
        2004 => reports(mode::BRACKETPASTE),
        2026 => reports(mode::SYNC),
        2031 => reports(mode::THEME_UPDATES),
        _ => None,
    };
    match set {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    }
}

/// tmux's DECRQM answer for a standard mode: 1 set, 2 reset, 0 unrecognized.
fn ansi_mode_status(modes: u32, mode_number: u32) -> u8 {
    match mode_number {
        4 if modes & mode::INSERT != 0 => 1,
        4 => 2,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::OutputPolicy;

    fn process(terminal: &mut Terminal, bytes: &[u8]) -> Vec<TerminalEvent> {
        terminal.process(bytes, &OutputPolicy::default())
    }

    #[test]
    fn a_cursor_report_captures_the_cursor_at_its_point_in_the_stream() {
        let mut terminal = Terminal::new(80, 24);
        let events = process(&mut terminal, b"AB\x1b[6nXY");
        assert_eq!(events, [TerminalEvent::Reply(b"\x1b[1;3R".to_vec())]);
        assert_eq!(terminal.screen().cursor_position(), (4, 0));
    }

    #[test]
    fn a_size_report_answers_in_cells_and_in_the_pushed_pixel_geometry() {
        let mut terminal = Terminal::new(80, 24);
        assert_eq!(
            process(&mut terminal, b"\x1b[18t\x1b[14t"),
            [
                TerminalEvent::Reply(b"\x1b[8;24;80t".to_vec()),
                // 24 rows x 32 px, 80 cols x 16 px: tmux's default cell size,
                // in force until the server pushes a client's.
                TerminalEvent::Reply(b"\x1b[4;768;1280t".to_vec()),
            ]
        );
    }

    #[test]
    fn a_tab_stop_lands_in_the_column_the_cursor_was_in() {
        let mut terminal = Terminal::new(80, 24);
        let events = process(&mut terminal, b"ABC\x1bHD");
        assert_eq!(events, []);
        assert!(terminal.screen().tab_stops().contains(&3));
    }

    #[test]
    fn a_mode_report_answers_from_the_mode_word_before_later_changes() {
        let mut terminal = Terminal::new(80, 24);
        // Query bracketed paste, then enable it afterwards in the same chunk:
        // the answer is "reset", read at the query's point in the stream.
        let events = process(&mut terminal, b"\x1b[?2004$p\x1b[?2004h");
        assert_eq!(events, [TerminalEvent::Reply(b"\x1b[?2004;2$y".to_vec())]);
        assert_ne!(terminal.screen().modes() & mode::BRACKETPASTE, 0);
    }

    #[test]
    fn a_decrqss_answer_reads_the_style_the_screen_holds_here() {
        let mut terminal = Terminal::new(80, 24);
        // Set a steady underline, query it, then change to a bar afterwards in
        // the same chunk: the answer reports the underline.
        let events = process(&mut terminal, b"\x1b[4 q\x1bP$q q\x1b\\\x1b[6 q");
        assert_eq!(events, [TerminalEvent::Reply(b"\x1bP1$r q4 q\x1b\\".to_vec())]);
        assert_eq!(terminal.screen().cursor_style(), 6);
    }

    #[test]
    fn an_alternate_mode_report_reads_the_switch_the_screen_made() {
        let mut terminal = Terminal::new(80, 24);
        let events = process(&mut terminal, b"\x1b[?1049h\x1b[?1049$p");
        assert!(terminal.screen().alternate_active());
        assert_eq!(
            events.last(),
            Some(&TerminalEvent::Reply(b"\x1b[?1049;1$y".to_vec()))
        );
    }

    #[test]
    fn the_title_lives_on_the_screen_and_the_stack_pushes_and_pops() {
        let mut terminal = Terminal::new(80, 24);
        assert_eq!(terminal.screen().title(), None);
        process(&mut terminal, b"\x1b]2;first\x07\x1b[22;0t\x1b]0;second\x07");
        assert_eq!(terminal.screen().title(), Some("second".to_string()));
        process(&mut terminal, b"\x1b[23;0t");
        assert_eq!(
            terminal.screen().title(),
            Some("first".to_string()),
            "the pop restores what the push saved"
        );
        // An empty stack leaves the title alone.
        process(&mut terminal, b"\x1b[23;2t");
        assert_eq!(terminal.screen().title(), Some("first".to_string()));
    }

    #[test]
    fn a_refused_title_leaves_the_screen_title_alone() {
        let mut terminal = Terminal::new(80, 24);
        let refused = OutputPolicy {
            allow_set_title: false,
            ..OutputPolicy::default()
        };
        terminal.process(b"\x1b]2;secret\x07", &refused);
        assert_eq!(terminal.screen().title(), None);
    }

    #[test]
    fn a_full_screen_scroll_ends_the_events_with_large_scroll() {
        let mut terminal = Terminal::new(80, 4);
        let events = process(&mut terminal, b"\n\n\n\n\n\x07");
        assert_eq!(events, [TerminalEvent::Bell, TerminalEvent::LargeScroll]);
    }

    #[test]
    fn expire_ground_reports_whether_a_sequence_was_abandoned() {
        let mut terminal = Terminal::new(80, 24);
        assert!(!terminal.expire_ground());
        process(&mut terminal, b"\x1b]0;half a title");
        assert!(terminal.awaiting_terminator());
        assert!(terminal.expire_ground());
        assert!(!terminal.awaiting_terminator());
    }

    #[test]
    fn reset_returns_the_scroll_region_to_the_whole_screen() {
        let mut terminal = Terminal::new(80, 24);
        process(&mut terminal, b"\x1b[5;10r");
        assert_eq!(terminal.scroll_region(), (4, 9));
        terminal.reset();
        assert_eq!(terminal.scroll_region(), (0, 23));
    }
}

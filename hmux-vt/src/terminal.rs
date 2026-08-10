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

use crate::observer::{Event, Observer, OutputPolicy};
use crate::parser::{tokenize, StringEnd, Token};
use crate::screen::{mode, PaneScreen};
use crate::scroll::ScrollRedraw;
use crate::{ClipboardEvent, OscUpdate};

/// Something in the processed byte stream the server has to act on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalEvent {
    /// A `BEL` reached the screen.
    Bell,
    /// The pane retitled itself (OSC 0/2 or APC), and the option allowed it.
    Title(String),
    /// `CSI 22 ; 0|2 t`: put the title as it stands on the title stack.
    /// Unlike [`TerminalEvent::Title`] this is not gated on `allow-set-title`
    /// — tmux's `screen_push_title` is reached without consulting the option,
    /// and the pane is only saving a title it is already allowed to have.
    TitlePush,
    /// `CSI 23 ; 0|2 t`: take the title back off the stack. An empty stack
    /// leaves the title alone.
    TitlePop,
    /// `ESC k … ST`: the screen-family window rename control.
    Rename(String),
    /// The pane switched screens (DECSET 47 / 1047 / 1049).
    AlternateScreen(bool),
    /// DECSET 1049 is about to switch, and this is the cursor to remember —
    /// read before the switch, because the switch is what moves it.
    SaveAlternateCursor { x: u16, y: u16 },
    /// DSR 6n, and where the cursor stood when the question arrived.
    CursorPositionReport { x: u16, y: u16 },
    /// CSI window operation: a terminal size query, with the dimensions the
    /// screen had when it was asked.
    WindowSizeReport { operation: u32, cols: u16, rows: u16 },
    /// DECRQM, with the screen's mode word as it stood at the question's point
    /// in the stream.
    DecPrivateModeReport { mode: u32, screen_modes: u32 },
    /// ANSI DECRQM, with the same captured mode word.
    DecModeReport { mode: u32, screen_modes: u32 },
    /// DECRQSS: answer this setting request. The request is carried unparsed
    /// because most of what answers it is server state; the one screen fact it
    /// needs — whether the cursor blinks — is captured here.
    StatusReport {
        request: Vec<u8>,
        cursor_blinking: bool,
    },
    /// HTS: set a tab stop in this column, the cursor's at the time.
    SetTabStop { column: u16 },
    /// TBC 0: clear the tab stop in this column, the cursor's at the time.
    ClearTabStop { column: u16 },
    /// TBC 3: clear every tab stop.
    ClearAllTabStops,
    /// DECSCUSR: the pane asked for a cursor style.
    CursorShape(u8),
    /// Bytes to write back to the pane's own input.
    Reply(Vec<u8>),
    /// OSC 10 / 11 / 12 asked the pane for a colour it stores. The pane owns
    /// the colour values; this event preserves the query's number, position
    /// and terminator until that state is reached.
    ColourQuery { number: u32, end: StringEnd },
    /// Bytes to forward to the client's terminal, whose answer comes back to
    /// the pane.
    ForwardQuery(&'static [u8]),
    /// OSC 4 asked for a palette entry neither the pane nor `pane-colours`
    /// has. tmux asks the attached terminal instead and answers the pane with
    /// what comes back, so the index and the query's terminator are carried
    /// until the server can route it.
    PaletteQuery { index: u8, end: StringEnd },
    /// A pane colour or path the formats report.
    Osc(OscUpdate),
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

    /// The screen, for the operations that change it from outside the byte
    /// stream: options, the history limit, history trimming.
    pub fn screen_mut(&mut self) -> &mut PaneScreen {
        &mut self.screen
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
            let resolved = self.resolve(event);
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
    /// it reports at this point in the stream.
    fn resolve(&self, event: Event) -> TerminalEvent {
        match event {
            Event::Bell => TerminalEvent::Bell,
            Event::Title(title) => TerminalEvent::Title(title),
            Event::TitlePush => TerminalEvent::TitlePush,
            Event::TitlePop => TerminalEvent::TitlePop,
            Event::Rename(name) => TerminalEvent::Rename(name),
            Event::AlternateScreen(on) => TerminalEvent::AlternateScreen(on),
            Event::SaveAlternateCursor => {
                let (x, y) = self.screen.cursor_position();
                TerminalEvent::SaveAlternateCursor { x, y }
            }
            Event::CursorPositionReport => {
                let (x, y) = self.screen.cursor_position();
                TerminalEvent::CursorPositionReport { x, y }
            }
            Event::WindowSizeReport(operation) => {
                let dims = self.screen.grid_dims();
                TerminalEvent::WindowSizeReport {
                    operation,
                    cols: dims.cols,
                    rows: dims.viewport_rows,
                }
            }
            Event::DecPrivateModeReport(mode) => TerminalEvent::DecPrivateModeReport {
                mode,
                screen_modes: self.screen.modes(),
            },
            Event::DecModeReport(mode) => TerminalEvent::DecModeReport {
                mode,
                screen_modes: self.screen.modes(),
            },
            Event::StatusReport(request) => TerminalEvent::StatusReport {
                request,
                cursor_blinking: self.screen.modes() & mode::CURSOR_BLINKING != 0,
            },
            Event::SetTabStop => TerminalEvent::SetTabStop {
                column: self.screen.cursor_position().0,
            },
            Event::ClearTabStop => TerminalEvent::ClearTabStop {
                column: self.screen.cursor_position().0,
            },
            Event::ClearAllTabStops => TerminalEvent::ClearAllTabStops,
            Event::CursorShape(shape) => TerminalEvent::CursorShape(shape),
            Event::Reply(bytes) => TerminalEvent::Reply(bytes),
            Event::ColourQuery { number, end } => TerminalEvent::ColourQuery { number, end },
            Event::ForwardQuery(query) => TerminalEvent::ForwardQuery(query),
            Event::PaletteQuery { index, end } => TerminalEvent::PaletteQuery { index, end },
            Event::Osc(update) => TerminalEvent::Osc(update),
            Event::Clipboard(clipboard) => TerminalEvent::Clipboard(clipboard),
            Event::Passthrough(data) => TerminalEvent::Passthrough(data),
            Event::ThemeQuery => TerminalEvent::ThemeQuery,
        }
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
        assert_eq!(
            events,
            [TerminalEvent::CursorPositionReport { x: 2, y: 0 }]
        );
        assert_eq!(terminal.screen().cursor_position(), (4, 0));
    }

    #[test]
    fn a_tab_stop_captures_the_column_it_was_set_in() {
        let mut terminal = Terminal::new(80, 24);
        let events = process(&mut terminal, b"ABC\x1bHD");
        assert_eq!(events, [TerminalEvent::SetTabStop { column: 3 }]);
    }

    #[test]
    fn a_mode_report_captures_the_mode_word_before_later_changes() {
        let mut terminal = Terminal::new(80, 24);
        // Query bracketed paste, then enable it afterwards in the same chunk.
        let events = process(&mut terminal, b"\x1b[?2004$p\x1b[?2004h");
        let [TerminalEvent::DecPrivateModeReport { mode, screen_modes }] = events.as_slice()
        else {
            panic!("expected one mode report, got {events:?}");
        };
        assert_eq!(*mode, 2004);
        assert_eq!(screen_modes & mode::BRACKETPASTE, 0);
        assert_ne!(terminal.screen().modes() & mode::BRACKETPASTE, 0);
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

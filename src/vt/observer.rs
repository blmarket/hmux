//! What a pane's tokens *mean* to the server.
//!
//! [`Observer`] drives one [`Parser`] over a pane's output and turns the tokens
//! into two things:
//!
//! - the bytes the screen backend should see, with the sequences the pane's
//!   options refuse removed, and
//! - an ordered list of [`Event`]s: query replies, OSC state, mode changes,
//!   bells, clipboard, passthrough, titles.
//!
//! Every event carries the number of screen tokens that must be applied before
//! it. An event that reads the cursor — a DSR reply, a tab stop, the cursor
//! DECSET 1049 saves — is therefore delivered at exactly the point in the
//! stream where the cursor is what it needs to be, instead of at the end of a
//! read where the answer would already be stale.
//!
//! This replaces a battery of independent detectors, each of which recovered
//! escape-sequence framing from the raw byte stream on its own. There is one
//! framing now, and it is `input.c`'s.

use std::collections::VecDeque;

use super::parser::{Param, Parser, StringEnd, Token, TokenKind, INPUT_BUFFER_DEFAULT_SIZE};
use super::sixel;
use super::vis;
use super::x11_colour;

/// The OSC 11 question hmux forwards to the client's own terminal, because only
/// the outer terminal knows the answer.
pub(crate) const BACKGROUND_COLOR_QUERY: &[u8] = b"\x1b]11;?\x1b\\";

/// The DSR synchronization request Neovim appends to capability queries; the
/// outer terminal's `CSI 0 n` tells it every preceding reply has arrived.
pub(crate) const DEVICE_STATUS_REPORT_QUERY: &[u8] = b"\x1b[5n";

/// The version XTVERSION reports. hmux presents a tmux-compatible surface, and
/// an application that special-cases a terminal by name has to see the same
/// answer the daemon's command language claims to implement.
const XTVERSION_NAME: &str = "tmux";

/// The options a pane's *own output* has to be parsed against.
///
/// tmux reads these from `wp->options` inside `input_parse`, which runs with the
/// whole server in reach. hmux parses a pane's bytes away from the server state,
/// so the resolved values are pushed to the pane instead and re-pushed whenever
/// they can have changed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OutputPolicy {
    /// `alternate-screen`: whether smcup and rmcup switch screens at all.
    pub(crate) alternate_screen: bool,
    /// `allow-set-title`: whether the pane may retitle itself.
    pub(crate) allow_set_title: bool,
    /// `allow-passthrough`: how far a `DCS tmux;` payload reaches.
    pub(crate) passthrough: PassthroughPolicy,
    /// `input-buffer-size`: how long a terminal string may grow before the
    /// parser abandons it.
    pub(crate) input_buffer_size: u32,
    /// The resolved `cursor-style` option, as a DECSCUSR parameter. A pane
    /// that has not sent DECSCUSR uses this as its DECRQSS answer.
    pub(crate) cursor_style: u8,
    /// `pane-colours`, packed as `0xrrggbb` by index — the palette a query
    /// falls back to when the pane has set nothing itself.
    pub(crate) palette: Vec<Option<u32>>,
}

/// The options' own defaults, which a pane parses against until the server's
/// first refresh reaches it.
impl Default for OutputPolicy {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            allow_set_title: true,
            passthrough: PassthroughPolicy::Off,
            input_buffer_size: INPUT_BUFFER_DEFAULT_SIZE,
            cursor_style: 0,
            palette: Vec::new(),
        }
    }
}

/// `allow-passthrough`: whether a pane may write to a client's terminal
/// directly, and whether it has to be the pane on screen to do so.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PassthroughPolicy {
    #[default]
    Off,
    /// `on`: only clients whose current window holds the pane.
    Visible,
    /// `all`: also clients that merely have the window linked, which is tmux's
    /// `TTY_CTX_INVISIBLE_PANES`.
    Always,
}

/// One OSC 52 sequence seen in a pane's output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClipboardEvent {
    /// `OSC 52 ; <selection> ; <base64>` — an application setting the
    /// clipboard, already decoded.
    Set { data: Vec<u8> },
    /// `OSC 52 ; <selection> ; ?` — an application asking for it.
    Query {
        /// The selection character echoed back in the reply; empty when the
        /// request named none that tmux recognises.
        selection: String,
        /// Whether the request ended with ST rather than BEL, which the reply
        /// mirrors.
        string_terminator: bool,
    },
}

/// The cursor style a pane asked for with DECSCUSR.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CursorShape {
    #[default]
    Default,
    Block,
    Underline,
    Bar,
}

impl CursorShape {
    /// tmux's `screen_set_cursor_style` mapping of a DECSCUSR parameter.
    pub(crate) fn from_parameter(parameter: u8) -> Self {
        match parameter {
            1 | 2 => Self::Block,
            3 | 4 => Self::Underline,
            5 | 6 => Self::Bar,
            _ => Self::Default,
        }
    }

    /// The name `#{cursor_shape}` reports.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Block => "block",
            Self::Underline => "underline",
            Self::Bar => "bar",
        }
    }
}

/// One OSC sequence that changed a pane's reported state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OscUpdate {
    /// OSC 11 / 111: `#{pane_bg}`.
    Background(String),
    /// OSC 10 / 110: `#{pane_fg}`.
    Foreground(String),
    /// OSC 12 / 112: `#{cursor_colour}`.
    CursorColour(String),
    /// OSC 7: `#{pane_path}`, kept verbatim as tmux keeps it.
    Path(String),
    /// OSC 9;4: `#{pane_pb_state}` and `#{pane_pb_progress}`. The progress is
    /// absent when the report named only a state, which leaves the old value.
    ProgressBar { state: u8, progress: Option<u8> },
}

/// Something in a pane's output the server has to act on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Event {
    /// A `BEL` reached the screen.
    Bell,
    /// The pane retitled itself (OSC 0/2 or APC), and the option allowed it.
    Title(String),
    /// `CSI 22 ; 0|2 t`: put the title as it stands on the title stack.
    /// Unlike [`Event::Title`] this is not gated on `allow-set-title` — tmux's
    /// `screen_push_title` is reached without consulting the option, and the
    /// pane is only saving a title it is already allowed to have.
    TitlePush,
    /// `CSI 23 ; 0|2 t`: take the title back off the stack. An empty stack
    /// leaves the title alone.
    TitlePop,
    /// `ESC k … ST`: the screen-family window rename control.
    Rename(String),
    /// The pane switched screens (DECSET 47 / 1047 / 1049).
    AlternateScreen(bool),
    /// DECSET 1049 is about to switch: remember the cursor first, because the
    /// switch is what moves it.
    SaveAlternateCursor,
    /// DSR 6n: answer with where the cursor is now.
    CursorPositionReport,
    /// CSI window operation: answer a terminal size query from the pane's
    /// current dimensions.
    WindowSizeReport(u32),
    /// DECRQM: answer whether this private mode is set, as the screen has it
    /// at this point in the stream.
    DecPrivateModeReport(u32),
    /// ANSI DECRQM: answer whether this standard mode is set, as the screen
    /// has it at this point in the stream.
    DecModeReport(u32),
    /// DECRQSS: answer this setting request. The payload is the request itself,
    /// still unparsed, because what answers it is state the observer does not
    /// hold.
    StatusReport(Vec<u8>),
    /// HTS: set a tab stop in the cursor's column.
    SetTabStop,
    /// TBC 0: clear the tab stop in the cursor's column.
    ClearTabStop,
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
}

/// One pass of [`Observer::feed`].
#[derive(Debug, Default)]
pub(crate) struct Observed {
    /// The tokens the screen should apply, in order. A sequence the pane's
    /// options refuse is absent; one the options rewrite appears rewritten,
    /// bytes and all.
    pub(crate) screen: Vec<Token>,
    /// Events, each paired with how many of `screen`'s tokens precede it.
    pub(crate) events: Vec<(usize, Event)>,
}

/// One pane's tokenizer plus the state the tokens change.
pub(crate) struct Observer {
    parser: Parser,
    /// The pane's `OSC 4` palette, as packed `0xrrggbb`. tmux keeps one per
    /// pane so a query is answered from what that pane set, not the client's.
    palette: Box<[Option<u32>; 256]>,
}

impl Default for Observer {
    fn default() -> Self {
        Self {
            parser: Parser::default(),
            palette: Box::new([None; 256]),
        }
    }
}

impl Observer {
    /// The bytes of a sequence the tokenizer has not finished, which
    /// `capture-pane -P` returns. See [`Parser::pending`].
    pub(crate) fn pending(&self) -> &[u8] {
        self.parser.pending()
    }

    /// Tokenize one chunk of pane output against the current options.
    pub(crate) fn feed(&mut self, input: &[u8], policy: &OutputPolicy) -> Observed {
        self.parser.set_string_capacity(policy.input_buffer_size);
        let mut tokens = VecDeque::new();
        self.parser.parse(input, |token| tokens.push_back(token));
        let mut observed = Observed::default();
        while let Some(token) = tokens.pop_front() {
            self.token(token, policy, &mut observed);
        }
        observed
    }

    fn token(&mut self, token: Token, policy: &OutputPolicy, out: &mut Observed) {
        match token.kind.clone() {
            // The decoded character, not the bytes it came from: a malformed
            // sequence has already been resolved to one replacement here, and
            // handing the original bytes on would let a backend that parses for
            // itself resolve it again, differently.
            TokenKind::Print(character) => {
                let mut buffer = [0u8; 4];
                let bytes = character.encode_utf8(&mut buffer).as_bytes().to_vec();
                out.rewrite(TokenKind::Print(character), bytes);
            }
            // An unfinished UTF-8 character contributes no bytes: the ones it
            // has collected travel with the token it eventually resolves into.
            TokenKind::Utf8Started => {
                out.rewrite(TokenKind::Utf8Started, Vec::new());
            }
            // Likewise for the replacement a malformed sequence resolved to:
            // the bytes handed on are the replacement's, not the bad ones.
            TokenKind::Replacement => {
                out.rewrite(TokenKind::Replacement, "\u{fffd}".as_bytes().to_vec());
            }
            TokenKind::Control(byte) => {
                if byte == 0x07 {
                    out.event(Event::Bell);
                }
                out.keep(token);
            }
            TokenKind::Esc {
                intermediates,
                final_byte,
            } => self.esc(token, &intermediates, final_byte, out),
            TokenKind::Csi {
                private,
                params,
                intermediates,
                final_byte,
            } => self.csi(
                token,
                private,
                &params,
                &intermediates,
                final_byte,
                policy,
                out,
            ),
            TokenKind::Osc { data, end } => self.osc(token, &data, end, policy, out),
            TokenKind::Dcs {
                intermediates,
                final_byte,
                data,
                ..
            } => self.dcs(token, &intermediates, final_byte, &data, policy, out),
            TokenKind::Apc { data } => {
                // tmux hands an APC title to the same `screen_set_title` as OSC
                // 0/2, and an emulator that does not recognize APC would print
                // it. Rewriting it as the OSC 2 it means keeps the two in
                // stream order, so the last one to arrive still wins.
                if policy.allow_set_title {
                    if let Some(title) = vis::clean_name(&data) {
                        out.event(Event::Title(title));
                    }
                    let mut body = b"2;".to_vec();
                    body.extend_from_slice(&data);
                    let mut raw = b"\x1b]".to_vec();
                    raw.extend_from_slice(&body);
                    raw.extend_from_slice(b"\x1b\\");
                    out.rewrite(
                        TokenKind::Osc {
                            data: body,
                            end: StringEnd::StringTerminator,
                        },
                        raw,
                    );
                }
            }
            TokenKind::Rename { data } => {
                // The screen/tmux rename control. Nothing downstream recognizes
                // it, so it is reported here and dropped from the stream rather
                // than printed as literal text.
                out.event(Event::Rename(String::from_utf8_lossy(&data).into_owned()));
            }
        }
    }

    fn esc(&mut self, token: Token, intermediates: &[u8], final_byte: u8, out: &mut Observed) {
        // HTS is the one escape the server has to see: the stop lands in the
        // column the cursor is in *now*, so the screen has to be told where
        // that is rather than working it out at the end of the read.
        if intermediates.is_empty() && final_byte == b'H' {
            out.event(Event::SetTabStop);
        }
        out.keep(token);
    }

    #[allow(clippy::too_many_arguments)]
    fn csi(
        &mut self,
        token: Token,
        private: Option<u8>,
        params: &[Param],
        intermediates: &[u8],
        final_byte: u8,
        policy: &OutputPolicy,
        out: &mut Observed,
    ) {
        match (private, intermediates, final_byte) {
            // DECSET / DECRST.
            (Some(b'?'), [], b'h' | b'l') => {
                return self.dec_mode(token, params, final_byte == b'h', policy, out);
            }
            // DECRQM for a private mode. The answer is a mode word read, so
            // it is ordered into the stream like the cursor report: what the
            // pane asked about is what the screen has *here*. tmux answers only
            // a request that names a mode — `if (m > 0)` — so a bare or zero
            // parameter produces nothing at all.
            (Some(b'?'), [b'$'], b'p') => {
                if let Some(mode) = params.first().map_or(Some(0), |param| param.get(0)) {
                    if mode > 0 {
                        out.event(Event::DecPrivateModeReport(mode));
                    }
                }
            }
            // ANSI DECRQM for a standard mode.
            (None, [b'$'], b'p') => {
                if let Some(mode) = params.first().map_or(Some(0), |param| param.get(0)) {
                    if mode > 0 {
                        out.event(Event::DecModeReport(mode));
                    }
                }
            }
            // Secondary DA and XTVERSION.
            (Some(b'>'), [], b'c') if first_is_zero(params) => {
                // 84 is `T`, tmux's terminal identifier.
                out.event(Event::Reply(b"\x1b[>84;0;0c".to_vec()));
            }
            (Some(b'>'), [], b'q') if first_is_zero(params) => {
                out.event(Event::Reply(
                    format!("\x1bP>|{XTVERSION_NAME} {}\x1b\\", super::TMUX_VERSION).into_bytes(),
                ));
            }
            // XTSMGRAPHICS, tmux's `input_csi_dispatch_sm_graphics`. The
            // daemon does not render sixel image payloads, but tmux answers
            // this capability query locally and applications use it
            // independently of image rendering.
            //
            // Only a request for the colour-register count reports the
            // register count; every other item — including one that names no
            // item at all — is answered with the failure form, which is why an
            // unrecognised `CSI ? … S` still produces a reply.
            (Some(b'?'), [], b'S') if params.len() <= 3 => {
                // tmux's `input_get(ictx, i, 0, 0)`: a position left out or
                // left empty reads as zero, and one carrying sub-parameters as
                // -1, which its `%d` prints rather than suppressing the reply.
                let read = |index: usize| -> i64 {
                    params
                        .get(index)
                        .map_or(0, |param| param.get(0).map_or(-1, i64::from))
                };
                let (item, action, value) = (read(0), read(1), read(2));
                let reply = if item == 1 && matches!(action, 1 | 2 | 4) {
                    format!("\x1b[?{item};0;{}S", sixel::COLOUR_REGISTERS)
                } else {
                    format!("\x1b[?{item};3;{value}S")
                };
                out.event(Event::Reply(reply.into_bytes()));
            }
            // DSR ?996: which theme is this terminal using?
            (Some(b'?'), [], b'n') if params.first().is_some_and(|p| p.get(0) == Some(996)) => {
                out.event(Event::ThemeQuery);
            }
            // Primary DA.
            (None, [], b'c') if first_is_zero(params) => {
                // The pinned tmux is built with sixel support, which is what
                // puts the `4` in its answer.
                out.event(Event::Reply(b"\x1b[?1;2;4c".to_vec()));
            }
            (None, [], b'n') => match params.first().map_or(Some(0), |param| param.get(0)) {
                // DSR 5n: no malfunction — and the outer terminal is asked the
                // same question, so a pane waiting on the round trip is
                // released in stream order.
                Some(5) => {
                    out.event(Event::Reply(b"\x1b[0n".to_vec()));
                    out.event(Event::ForwardQuery(DEVICE_STATUS_REPORT_QUERY));
                }
                // DSR 6n is answered with the cursor as it stands here, which
                // is why the event is ordered inside the stream at all.
                Some(6) => out.event(Event::CursorPositionReport),
                _ => {}
            },
            // CSI window operations that report the pane's dimensions. The
            // pixel geometry uses tmux's default cell size when no attached
            // terminal has supplied one.
            (None, [], b't') => window_operations(params, out),
            // TBC.
            (None, [], b'g') => match params.first().map_or(Some(0), |param| param.get(0)) {
                Some(0) => out.event(Event::ClearTabStop),
                Some(3) => out.event(Event::ClearAllTabStops),
                _ => {}
            },
            // DECSCUSR.
            (None, [b' '], b'q') => {
                if let Some(shape) = params.first().map_or(Some(0), |param| param.get(0)) {
                    if shape <= 6 {
                        out.event(Event::CursorShape(shape as u8));
                    }
                }
            }
            _ => {}
        }
        out.keep(token);
    }

    /// DECSET/DECRST: apply each parameter in turn, as `input.c` does, and drop
    /// the screen switches the `alternate-screen` option refuses.
    fn dec_mode(
        &mut self,
        token: Token,
        params: &[Param],
        set: bool,
        policy: &OutputPolicy,
        out: &mut Observed,
    ) {
        let mut forwarded: Vec<u32> = Vec::with_capacity(params.len());
        for param in params {
            let Some(mode) = param.value else { continue };
            let alternate = matches!(mode, 47 | 1047 | 1049);
            // tmux parses an alternate-screen switch and then returns early
            // from `screen_write_alternateon`/`_alternateoff` when the option
            // is off, so the switch has no effect at all — including on the
            // cursor 1049 would have saved. Dropping the parameter before the
            // screen sees it is the same observable.
            if alternate && !policy.alternate_screen {
                continue;
            }
            if alternate {
                if mode == 1049 && set {
                    // The save has to happen before the switch runs, since
                    // moving screens is what moves the cursor away.
                    out.event(Event::SaveAlternateCursor);
                }
                out.event(Event::AlternateScreen(set));
            }
            forwarded.push(mode);
        }
        if forwarded.len() == params.len() {
            out.keep(token);
            return;
        }
        if forwarded.is_empty() {
            return;
        }
        // Some parameters were refused: re-emit the sequence with the rest, so
        // an unrelated mode in the same sequence still applies.
        let list: Vec<String> = forwarded.iter().map(u32::to_string).collect();
        let mut raw = b"\x1b[?".to_vec();
        raw.extend_from_slice(list.join(";").as_bytes());
        raw.push(if set { b'h' } else { b'l' });
        out.rewrite(
            TokenKind::Csi {
                private: Some(b'?'),
                params: forwarded
                    .into_iter()
                    .map(|mode| Param {
                        value: Some(mode),
                        subs: Vec::new(),
                    })
                    .collect(),
                intermediates: Vec::new(),
                final_byte: if set { b'h' } else { b'l' },
            },
            raw,
        );
    }

    fn osc(
        &mut self,
        token: Token,
        data: &[u8],
        end: StringEnd,
        policy: &OutputPolicy,
        out: &mut Observed,
    ) {
        // tmux's `input_exit_osc`: a body that does not open with digits names
        // no command at all.
        let digits = data.iter().take_while(|byte| byte.is_ascii_digit()).count();
        if digits == 0 {
            out.keep(token);
            return;
        }
        let Ok(number) = std::str::from_utf8(&data[..digits])
            .unwrap_or_default()
            .parse::<u32>()
        else {
            out.keep(token);
            return;
        };
        let rest = &data[digits..];
        let body = match rest.first() {
            None => &[][..],
            Some(b';') => &rest[1..],
            // Anything else is not a well-formed OSC.
            Some(_) => {
                out.keep(token);
                return;
            }
        };
        let text = String::from_utf8_lossy(body);
        match number {
            0 | 2 => {
                if policy.allow_set_title {
                    // `screen_set_title` refuses a name it cannot clean, and a
                    // refused one leaves the title that was already set.
                    if let Some(title) = vis::clean_name(body) {
                        out.event(Event::Title(title));
                    }
                    out.keep(token);
                }
                // Refused: the sequence never reaches the screen at all.
                return;
            }
            4 => {
                self.palette_request(&text, end, policy, out);
            }
            // `screen_set_path` cleans a path by the same rule as a title.
            7 => {
                if let Some(path) = vis::clean_name(body) {
                    out.event(Event::Osc(OscUpdate::Path(path)));
                }
            }
            9 => {
                if let Some(update) = progress_bar_report(&text) {
                    out.event(Event::Osc(update));
                }
            }
            10..=12 => {
                if text == "?" {
                    // These colours are pane-local, so the pane answers at the
                    // same point in the stream. Only it can tell whether it has
                    // a value stored, so the fallback for an unset background —
                    // asking the outer terminal — is left to it too.
                    out.event(Event::ColourQuery { number, end });
                } else if let Some(colour) = parse_colour(&text) {
                    out.event(Event::Osc(match number {
                        10 => OscUpdate::Foreground(colour),
                        11 => OscUpdate::Background(colour),
                        _ => OscUpdate::CursorColour(colour),
                    }));
                }
            }
            52 => {
                if let Some(event) = clipboard_event(body, end) {
                    out.event(Event::Clipboard(event));
                }
            }
            104 => {
                if body.is_empty() {
                    *self.palette = [None; 256];
                } else {
                    for index in text.split(';') {
                        match index.parse::<u8>() {
                            Ok(index) => self.palette[usize::from(index)] = None,
                            // tmux stops at the first index it cannot read.
                            Err(_) => break,
                        }
                    }
                }
            }
            // The reset forms. tmux spells an unset cursor colour `none` but an
            // unset foreground or background `default`.
            110 => out.event(Event::Osc(OscUpdate::Foreground("default".to_string()))),
            111 => out.event(Event::Osc(OscUpdate::Background("default".to_string()))),
            112 => out.event(Event::Osc(OscUpdate::CursorColour("none".to_string()))),
            _ => {}
        }
        out.keep(token);
    }

    /// Apply one `OSC 4` body, which is a run of `index ; value` pairs.
    ///
    /// A `?` value asks for the entry back. tmux answers from its own palette
    /// when the entry has been set and otherwise forwards the question to the
    /// client's terminal; with nothing stored here the question is dropped,
    /// which is what keeps an unset entry silent.
    fn palette_request(
        &mut self,
        body: &str,
        end: StringEnd,
        policy: &OutputPolicy,
        out: &mut Observed,
    ) {
        let mut reply = Vec::new();
        let mut rest = body;
        while !rest.is_empty() {
            let Some((index, tail)) = rest.split_once(';') else {
                break;
            };
            // tmux stops at the first unparseable index rather than skipping it.
            let Ok(index) = index.parse::<u8>() else {
                break;
            };
            let (value, tail) = match tail.split_once(';') {
                Some((value, tail)) => (value, tail),
                None => (tail, ""),
            };
            if value == "?" {
                let option = policy.palette.get(usize::from(index)).copied().flatten();
                if let Some(colour) = self.palette[usize::from(index)].or(option) {
                    let (r, g, b) = ((colour >> 16) as u8, (colour >> 8) as u8, colour as u8);
                    let terminator = match end {
                        StringEnd::StringTerminator => "\x1b\\",
                        StringEnd::Bell => "\x07",
                    };
                    // tmux answers in 16-bit components, each byte doubled.
                    reply.extend_from_slice(
                        format!(
                            "\x1b]4;{index};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}\
                             {terminator}"
                        )
                        .as_bytes(),
                    );
                } else {
                    // Nothing here holds this entry: the question is for the
                    // terminal, and its answer for the pane.
                    out.event(Event::PaletteQuery { index, end });
                }
            } else if let Some(packed) = parse_packed_colour(value) {
                self.palette[usize::from(index)] = Some(packed);
            }
            rest = tail;
        }
        if !reply.is_empty() {
            out.event(Event::Reply(reply));
        }
    }

    fn dcs(
        &mut self,
        token: Token,
        intermediates: &[u8],
        final_byte: u8,
        data: &[u8],
        policy: &OutputPolicy,
        out: &mut Observed,
    ) {
        // DECRQSS is `DCS $ q Pt ST`. tmux takes it before anything else with a
        // `$` intermediate.
        if intermediates == [b'$'] && final_byte == b'q' {
            // The cursor style and its blink are the pane's and the screen's
            // respectively, so the reply is composed where both are in reach.
            out.event(Event::StatusReport(data.to_vec()));
            return;
        }
        // A sixel image is the one DCS that reaches the grid — it scrolls the
        // screen to make room for itself and moves the cursor — so it is the
        // one the screen is given. tmux's test is `buf[0] == 'q'` with no
        // intermediate, which here is the final byte with no intermediate.
        if final_byte == b'q' && intermediates.is_empty() {
            out.keep(token);
            return;
        }
        // `DCS tmux; … ST`. The final byte is the first byte of tmux's DCS
        // string, so the prefix straddles the two.
        // Anything else tmux ignores, and so does the screen: a DCS carries no
        // grid content hmux reproduces, so nothing is forwarded either.
        if final_byte == b't'
            && data.starts_with(b"mux;")
            && policy.passthrough != PassthroughPolicy::Off
        {
            out.event(Event::Passthrough(data[4..].to_vec()));
        }
    }
}

impl Observed {
    fn event(&mut self, event: Event) {
        let at = self.screen.len();
        self.events.push((at, event));
    }

    /// Pass a token through to the screen unchanged.
    fn keep(&mut self, token: Token) {
        self.screen.push(token);
    }

    /// Hand the screen a token the observer built, with the bytes an emulator
    /// that parses for itself would need to see.
    fn rewrite(&mut self, kind: TokenKind, raw: Vec<u8>) {
        self.screen.push(Token { kind, raw });
    }
}

/// XTWINOPS, as tmux's `input_csi_dispatch_winops` walks it.
///
/// The parameters are a stream of operations rather than a list of them: an
/// operation that takes arguments consumes the positions after it, and a
/// position that is not there — or is empty, or carries sub-parameters — ends
/// the walk where it stands. Everything tmux does not implement is read and
/// dropped, silently: only the geometry reports answer, so a resize request
/// like `CSI 8 ; rows ; cols t` writes nothing back to the pane.
fn window_operations(params: &[Param], out: &mut Observed) {
    // `input_get(ictx, m, 0, -1)`: an absent, empty or sub-parametered position
    // is the -1 that stops the loop, not a defaulted zero.
    let read = |index: usize| -> Option<u32> {
        params
            .get(index)
            .filter(|param| !param.is_string())
            .and_then(|param| param.value)
    };
    let mut index = 0;
    while let Some(operation) = read(index) {
        match operation {
            // Implemented as "understood, and nothing to do".
            1 | 2 | 5 | 6 | 7 | 11 | 13 | 20 | 21 | 24 => {}
            // Two arguments: move, resize in pixels, resize in cells.
            3 | 4 | 8 => {
                index += 2;
                if read(index - 1).is_none() || read(index).is_none() {
                    return;
                }
            }
            // One argument: raise/lower, refresh.
            9 | 10 => {
                index += 1;
                if read(index).is_none() {
                    return;
                }
            }
            // The geometry reports, which are all this answers.
            14 | 15 | 16 | 18 | 19 => out.event(Event::WindowSizeReport(operation)),
            // Push and pop the title, one argument saying which title. Only
            // the window title and the "both" spelling reach the stack; the
            // icon-name-only form is understood and does nothing. A missing
            // argument abandons the rest of the sequence, as `case -1` does.
            22 | 23 => {
                index += 1;
                match read(index) {
                    None => return,
                    Some(0 | 2) if operation == 22 => out.event(Event::TitlePush),
                    Some(0 | 2) => out.event(Event::TitlePop),
                    Some(_) => {}
                }
            }
            _ => {}
        }
        index += 1;
    }
}

/// Whether the sequence's first parameter is absent or zero — the only forms
/// tmux's device-attribute handlers answer.
fn first_is_zero(params: &[Param]) -> bool {
    params.first().map_or(Some(0), |param| param.get(0)) == Some(0)
}

/// tmux's `input_handle_decrqss` reply. The only setting it reports is the
/// cursor style; anything else gets `DCS 0 $ r ST`, its "invalid request" form.
pub(crate) fn decrqss_reply(
    request: &[u8],
    shape: CursorShape,
    blinking: bool,
    configured_style: u8,
) -> Vec<u8> {
    if request != b" q" {
        return b"\x1bP0$r\x1b\\".to_vec();
    }
    let ps = match shape {
        CursorShape::Default => configured_style,
        CursorShape::Block if blinking => 1,
        CursorShape::Block => 2,
        CursorShape::Underline if blinking => 3,
        CursorShape::Underline => 4,
        CursorShape::Bar if blinking => 5,
        CursorShape::Bar => 6,
    };
    format!("\x1bP1$r q{ps} q\x1b\\").into_bytes()
}

/// Parse an `OSC 9` body as tmux's `input_osc_9` does.
///
/// Only the `4` subcommand — the ConEmu progress report — means anything to
/// tmux. A report naming just a state leaves the previous progress in place,
/// which is why the value is optional.
fn progress_bar_report(body: &str) -> Option<OscUpdate> {
    let rest = body.strip_prefix('4')?;
    // `9;4` and `9;4;` carry no state and are dropped rather than reset.
    let rest = rest.strip_prefix(';').filter(|rest| !rest.is_empty())?;
    let (state, rest) = rest.split_at_checked(1)?;
    let state = state.parse::<u8>().ok().filter(|state| *state <= 4)?;
    let Some(progress) = rest.strip_prefix(';').filter(|rest| !rest.is_empty()) else {
        // Anything other than a clean end here is malformed, not a bare state.
        return rest.is_empty().then_some(OscUpdate::ProgressBar {
            state,
            progress: None,
        });
    };
    let progress = progress.parse::<u8>().ok().filter(|value| *value <= 100)?;
    Some(OscUpdate::ProgressBar {
        state,
        progress: Some(progress),
    })
}

/// One `OSC 52` body, as tmux's `input_osc_52` reads it.
fn clipboard_event(body: &[u8], end: StringEnd) -> Option<ClipboardEvent> {
    let text = std::str::from_utf8(body).ok()?;
    // A sequence with no `;` at all names no payload and is dropped.
    let (selection, payload) = text.split_once(';')?;
    if payload == "?" {
        return Some(ClipboardEvent::Query {
            selection: osc52_reply_selection(selection),
            string_terminator: end == StringEnd::StringTerminator,
        });
    }
    // Empty or undecodable data is dropped, as tmux drops it.
    if payload.is_empty() {
        return None;
    }
    base64_decode_strict(payload).map(|data| ClipboardEvent::Set { data })
}

/// The selection tmux echoes back in an OSC 52 reply: the first character it
/// recognises, or nothing when the request named none.
fn osc52_reply_selection(selection: &str) -> String {
    selection
        .chars()
        .find(|character| "cpqs01234567".contains(*character))
        .map(String::from)
        .unwrap_or_default()
}

/// Decode standard base64, rejecting anything that is not fully padded — which
/// is what makes tmux drop `aGk` where it accepts `aGk=`.
pub(crate) fn base64_decode_strict(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = text.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2 {
        return None;
    }
    let data = &bytes[..bytes.len() - padding];
    let mut out = Vec::with_capacity(data.len() / 4 * 3);
    let (mut accumulator, mut bits) = (0u32, 0u32);
    for byte in data {
        let position = ALPHABET.iter().position(|candidate| candidate == byte)?;
        accumulator = (accumulator << 6) | position as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

/// An X11 colour payload as a packed `0xrrggbb`, for the palette store.
pub(crate) fn parse_packed_colour(value: &str) -> Option<u32> {
    let text = parse_colour(value)?;
    u32::from_str_radix(text.strip_prefix('#')?, 16).ok()
}

/// An X11 colour specification as the `#rrggbb` string the formats report.
pub(crate) fn parse_colour(value: &str) -> Option<String> {
    let value = value.trim();
    let rgb = if let Some(parts) = value.strip_prefix("rgb:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            scale_hex(parts[0])?,
            scale_hex(parts[1])?,
            scale_hex(parts[2])?,
        )
    } else if let Some(parts) = value.strip_prefix("cmy:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            ((1.0 - parts[0].parse::<f64>().ok()?) * 255.0) as u8,
            ((1.0 - parts[1].parse::<f64>().ok()?) * 255.0) as u8,
            ((1.0 - parts[2].parse::<f64>().ok()?) * 255.0) as u8,
        )
    } else if let Some(parts) = value.strip_prefix("cmyk:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 4 {
            return None;
        }
        let k = parts[3].parse::<f64>().ok()?;
        (
            ((1.0 - parts[0].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
            ((1.0 - parts[1].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
            ((1.0 - parts[2].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
        )
    } else if let Some(hex) = value.strip_prefix('#') {
        match hex.len() {
            6 => (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            ),
            12 => (
                scale_hex(&hex[0..4])?,
                scale_hex(&hex[4..8])?,
                scale_hex(&hex[8..12])?,
            ),
            _ => return None,
        }
    } else if value.contains(',') {
        let parts: Vec<_> = value.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        )
    } else {
        // Anything left is a name, which tmux resolves against X11's table
        // rather than the terminal palette — so `red` is #ff0000, not colour 1.
        let packed = x11_colour::colour_by_name(value)?;
        ((packed >> 16) as u8, (packed >> 8) as u8, packed as u8)
    };
    Some(format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2))
}

fn scale_hex(component: &str) -> Option<u8> {
    if component.is_empty() || component.len() > 4 {
        return None;
    }
    let value = u32::from_str_radix(component, 16).ok()?;
    if component.len() == 4 {
        return Some((value >> 8) as u8);
    }
    let maximum = 16u32.pow(component.len() as u32) - 1;
    Some(((value * 255 + maximum / 2) / maximum) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> OutputPolicy {
        OutputPolicy::default()
    }

    /// The bytes a backend that parses for itself would be handed: every
    /// token's own bytes, in order.
    fn screen_bytes(observed: &Observed) -> Vec<u8> {
        observed
            .screen
            .iter()
            .flat_map(|token| token.raw.iter().copied())
            .collect()
    }

    fn observe(input: &[u8]) -> (Observer, Observed) {
        let mut observer = Observer::default();
        let observed = observer.feed(input, &policy());
        (observer, observed)
    }

    fn events(input: &[u8]) -> Vec<Event> {
        observe(input)
            .1
            .events
            .into_iter()
            .map(|(_, e)| e)
            .collect()
    }

    #[test]
    fn ordinary_output_reaches_the_screen_unchanged() {
        let (_, observed) = observe(b"hello\r\n\x1b[31mred\x1b[0m");
        assert_eq!(screen_bytes(&observed), b"hello\r\n\x1b[31mred\x1b[0m");
        assert!(observed.events.is_empty());
    }

    #[test]
    fn a_bell_is_reported_but_not_inside_an_osc() {
        assert_eq!(events(b"\x07"), vec![Event::Bell]);
        assert!(events(b"\x1b]2;t\x07").iter().all(|e| *e != Event::Bell));
    }

    #[test]
    fn the_screen_rename_control_is_reported_and_removed() {
        let (_, observed) = observe(b"a\x1bkname\x1b\\b");
        assert_eq!(screen_bytes(&observed), b"ab");
        assert_eq!(
            observed.events,
            vec![(1, Event::Rename("name".to_string()))]
        );
    }

    #[test]
    fn an_apc_title_reaches_the_screen_as_the_osc_2_it_means() {
        let (_, observed) = observe(b"\x1b_title\x1b\\");
        assert_eq!(screen_bytes(&observed), b"\x1b]2;title\x1b\\");
        assert_eq!(
            observed.events,
            vec![(0, Event::Title("title".to_string()))]
        );
    }

    #[test]
    fn a_refused_title_never_reaches_the_screen() {
        let mut observer = Observer::default();
        let refused = OutputPolicy {
            allow_set_title: false,
            ..policy()
        };
        let observed = observer.feed(b"x\x1b]2;t\x07\x1b_a\x1b\\y", &refused);
        assert_eq!(screen_bytes(&observed), b"xy");
        assert!(observed.events.is_empty());
    }

    #[test]
    fn the_cursor_report_splits_the_stream_at_the_query() {
        let (_, observed) = observe(b"ab\x1b[6ncd");
        assert_eq!(screen_bytes(&observed), b"ab\x1b[6ncd");
        assert_eq!(
            observed.events,
            vec![(2, Event::CursorPositionReport)],
            "the reply must be built after ab and before cd, not at the end"
        );
    }

    #[test]
    fn the_alternate_screen_save_lands_before_the_switch() {
        let (_, observed) = observe(b"\x1b[?1049h");
        assert_eq!(
            observed.events,
            vec![
                (0, Event::SaveAlternateCursor),
                (0, Event::AlternateScreen(true)),
            ]
        );
        assert_eq!(screen_bytes(&observed), b"\x1b[?1049h");
    }

    #[test]
    fn a_refused_alternate_switch_is_dropped_but_its_neighbours_survive() {
        let mut observer = Observer::default();
        let refused = OutputPolicy {
            alternate_screen: false,
            ..policy()
        };
        let observed = observer.feed(b"\x1b[?1049h\x1b[?1049;1000h", &refused);
        assert_eq!(
            screen_bytes(&observed),
            b"\x1b[?1000h",
            "the 1049 goes, the mouse mode stays"
        );
        assert!(observed
            .events
            .iter()
            .all(|(_, event)| !matches!(event, Event::AlternateScreen(_))));
    }

    #[test]
    fn decrqm_asks_the_screen_rather_than_answering_itself() {
        // The observer no longer holds a mode word to answer from, so the query
        // becomes an event and is answered where the screen is in reach — at
        // this point in the stream, after the DECSET ahead of it has landed.
        assert_eq!(
            events(b"\x1b[?2026h\x1b[?2026$p"),
            vec![Event::DecPrivateModeReport(2026)]
        );
    }

    #[test]
    fn ansi_decrqm_is_reported_as_a_standard_mode_event() {
        assert_eq!(events(b"\x1b[4$p"), vec![Event::DecModeReport(4)]);
    }

    #[test]
    fn device_attribute_queries_are_answered_locally() {
        assert_eq!(
            events(b"\x1b[c"),
            vec![Event::Reply(b"\x1b[?1;2;4c".to_vec())]
        );
        assert_eq!(
            events(b"\x1b[>c"),
            vec![Event::Reply(b"\x1b[>84;0;0c".to_vec())]
        );
        assert_eq!(
            events(b"\x1b[5n"),
            vec![
                Event::Reply(b"\x1b[0n".to_vec()),
                Event::ForwardQuery(DEVICE_STATUS_REPORT_QUERY),
            ]
        );
        assert_eq!(
            events(b"\x1b[?1;1S"),
            vec![Event::Reply(b"\x1b[?1;0;1024S".to_vec())]
        );
    }

    #[test]
    fn a_colour_question_is_handed_to_the_pane_that_stores_the_colour() {
        for number in [10, 11, 12] {
            assert_eq!(
                events(format!("\x1b]{number};?\x07").as_bytes()),
                vec![Event::ColourQuery {
                    number,
                    end: StringEnd::Bell
                }]
            );
        }
    }

    #[test]
    fn osc_colours_and_paths_are_reported() {
        assert_eq!(
            events(b"\x1b]11;rgb:ff/00/00\x1b\\"),
            vec![Event::Osc(OscUpdate::Background("#ff0000".to_string()))]
        );
        assert_eq!(
            events(b"\x1b]7;file://h/tmp\x1b\\"),
            vec![Event::Osc(OscUpdate::Path("file://h/tmp".to_string()))]
        );
        assert_eq!(
            events(b"\x1b]112\x07"),
            vec![Event::Osc(OscUpdate::CursorColour("none".to_string()))]
        );
    }

    #[test]
    fn the_pane_palette_answers_its_own_query_with_the_requests_terminator() {
        assert_eq!(
            events(b"\x1b]4;1;#112233\x1b\\\x1b]4;1;?\x07"),
            vec![Event::Reply(b"\x1b]4;1;rgb:1111/2222/3333\x07".to_vec())]
        );
    }

    #[test]
    fn clipboard_sets_and_queries_are_reported() {
        assert_eq!(
            events(b"\x1b]52;c;aGk=\x07"),
            vec![Event::Clipboard(ClipboardEvent::Set {
                data: b"hi".to_vec()
            })]
        );
        assert_eq!(
            events(b"\x1b]52;c;?\x1b\\"),
            vec![Event::Clipboard(ClipboardEvent::Query {
                selection: "c".to_string(),
                string_terminator: true,
            })]
        );
    }

    #[test]
    fn passthrough_is_reported_only_when_the_option_allows_it() {
        assert!(events(b"\x1bPtmux;\x1b\x1b[31m\x1b\\").is_empty());
        let mut observer = Observer::default();
        let allowed = OutputPolicy {
            passthrough: PassthroughPolicy::Visible,
            ..policy()
        };
        let observed = observer.feed(b"\x1bPtmux;\x1b\x1b[31m\x1b\\", &allowed);
        assert_eq!(
            observed.events,
            vec![(0, Event::Passthrough(b"\x1b[31m".to_vec()))]
        );
        assert!(observed.screen.is_empty(), "a DCS never reaches the screen");
    }

    #[test]
    fn decrqss_is_forwarded_unparsed_for_the_pane_to_answer() {
        // The cursor style and the blink that answer this live on the pane and
        // the screen, so the observer hands the request on rather than holding
        // copies of both to answer it here.
        assert_eq!(
            events(b"\x1b[4 q\x1bP$q q\x1b\\"),
            vec![Event::CursorShape(4), Event::StatusReport(b" q".to_vec()),]
        );
    }

    #[test]
    fn a_decrqss_reply_reports_the_style_and_its_blink() {
        assert_eq!(
            decrqss_reply(b" q", CursorShape::Underline, false, 0),
            b"\x1bP1$r q4 q\x1b\\".to_vec()
        );
        assert_eq!(
            decrqss_reply(b"m", CursorShape::Default, false, 0),
            b"\x1bP0$r\x1b\\".to_vec(),
            "a setting hmux does not report is refused, not guessed at"
        );
        assert_eq!(
            decrqss_reply(b" q", CursorShape::Default, false, 6),
            b"\x1bP1$r q6 q\x1b\\".to_vec()
        );
    }

    #[test]
    fn tab_stop_edits_are_ordered_against_the_tokens_that_move_the_cursor() {
        let (_, observed) = observe(b"\x1b[5G\x1bH\x1b[g\x1b[3g");
        assert_eq!(
            observed.events,
            vec![
                (1, Event::SetTabStop),
                (2, Event::ClearTabStop),
                (3, Event::ClearAllTabStops),
            ],
            "each edit lands after the tokens that positioned the cursor for it"
        );
    }

    #[test]
    fn a_theme_question_is_reported_once() {
        assert_eq!(events(b"\x1b[?996n"), vec![Event::ThemeQuery]);
    }

    #[test]
    fn del_and_malformed_utf8_are_resolved_before_the_screen_sees_them() {
        let (_, observed) = observe(b"a\x7fb\xc3\x28");
        assert_eq!(
            String::from_utf8_lossy(&screen_bytes(&observed)),
            "ab\u{fffd}(",
            "one replacement per malformed sequence, DEL dropped"
        );
    }
}

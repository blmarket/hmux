//! What a pane's tokens *mean* to the server.
//!
//! [`Observer`] drives one [`Parser`] over a pane's output and turns the tokens
//! into two things:
//!
//! - the bytes the screen should see, with the sequences the pane's options
//!   refuse removed, and
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
//!
//! The grammar of a sequence lives with the module that dispatches it. That is
//! why the OSC 52 base64 rules ([`base64_decode_strict`]), the X11 colour
//! grammar ([`parse_packed_colour`]) and the DECRQSS answers
//! ([`decrqss_reply`]) are here: they parse and produce the payloads of
//! sequences this module handles, even when a caller reuses one from another
//! direction.

use super::parser::{INPUT_BUFFER_DEFAULT_SIZE, Param, Parser, StringEnd, Token, TokenKind};
use super::vis;
use super::x11_colour;

/// The OSC 11 question hmux forwards to the client's own terminal, because only
/// the outer terminal knows the answer.
pub const BACKGROUND_COLOR_QUERY: &[u8] = b"\x1b]11;?\x1b\\";

/// The DSR synchronization request Neovim appends to capability queries; the
/// outer terminal's `CSI 0 n` tells it every preceding reply has arrived.
const DEVICE_STATUS_REPORT_QUERY: &[u8] = b"\x1b[5n";

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
pub struct OutputPolicy {
    /// `alternate-screen`: whether smcup and rmcup switch screens at all.
    pub alternate_screen: bool,
    /// `allow-set-title`: whether the pane may retitle itself.
    pub allow_set_title: bool,
    /// `allow-passthrough`: how far a `DCS tmux;` payload reaches.
    pub passthrough: PassthroughPolicy,
    /// `input-buffer-size`: how long a terminal string may grow before the
    /// parser abandons it.
    pub input_buffer_size: u32,
    /// The resolved `cursor-style` option, as a DECSCUSR parameter. A pane
    /// that has not sent DECSCUSR uses this as its DECRQSS answer.
    pub cursor_style: u8,
    /// `pane-colours`, packed as `0xrrggbb` by index — the palette a query
    /// falls back to when the pane has set nothing itself.
    pub palette: Vec<Option<u32>>,
    /// `extended-keys`: whether a pane's `modifyOtherKeys` request is accepted
    /// at all. tmux decides this where the request is parsed, so a request made
    /// while the option was off stays refused after it is turned on.
    pub extended_keys: bool,
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
            extended_keys: true,
        }
    }
}

/// `allow-passthrough`: whether a pane may write to a client's terminal
/// directly, and whether it has to be the pane on screen to do so.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PassthroughPolicy {
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
pub enum ClipboardEvent {
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
pub enum CursorShape {
    #[default]
    Default,
    Block,
    Underline,
    Bar,
}

impl CursorShape {
    /// tmux's `screen_set_cursor_style` mapping of a DECSCUSR parameter.
    pub fn from_parameter(parameter: u8) -> Self {
        match parameter {
            1 | 2 => Self::Block,
            3 | 4 => Self::Underline,
            5 | 6 => Self::Bar,
            _ => Self::Default,
        }
    }

    /// The name `#{cursor_shape}` reports.
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Block => "block",
            Self::Underline => "underline",
            Self::Bar => "bar",
        }
    }
}

/// The pane state OSC sequences set, as the formats report it: one snapshot
/// of the observer's stored values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OscState {
    /// OSC 10 / 110: `#{pane_fg}`, `default` until the pane speaks.
    pub foreground: String,
    /// OSC 11 / 111: `#{pane_bg}`, `default` until the pane speaks.
    pub background: String,
    /// OSC 12 / 112: `#{cursor_colour}`. tmux spells the unset value `none`
    /// where the unset foreground and background are `default`.
    pub cursor_colour: String,
    /// OSC 7: `#{pane_path}`, kept verbatim as tmux keeps it.
    pub path: String,
    /// OSC 9;4: `#{pane_pb_state}`, `0..=4` as ConEmu numbers them.
    pub progress_state: u8,
    /// `#{pane_pb_progress}`, `0..=100`. A report naming only a state leaves
    /// the previous progress in place.
    pub progress_value: u8,
}

impl Default for OscState {
    fn default() -> Self {
        OscState {
            foreground: "default".to_string(),
            background: "default".to_string(),
            cursor_colour: "none".to_string(),
            path: String::new(),
            progress_state: 0,
            progress_value: 0,
        }
    }
}

/// Something in a pane's output the server has to act on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// A `BEL` reached the screen.
    Bell,
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
}

/// One pass of [`Observer::feed`].
#[derive(Debug, Default)]
pub struct Observed {
    /// The tokens the screen should apply, in order. A sequence the pane's
    /// options refuse is absent; one the options rewrite appears rewritten,
    /// bytes and all.
    pub screen: Vec<Token>,
    /// Events, each paired with how many of `screen`'s tokens precede it.
    pub events: Vec<(usize, Event)>,
}

/// One pane's tokenizer plus the state the tokens change.
pub struct Observer {
    parser: Parser,
    /// What a token changes, held apart from the tokenizer so a token can be
    /// observed while the tokenizer is still reading the chunk it came from.
    state: ObserverState,
}

/// The pane state the tokens change, which is everything the observer holds
/// except the tokenizer itself.
struct ObserverState {
    /// The pane's `OSC 4` palette, as packed `0xrrggbb`. tmux keeps one per
    /// pane so a query is answered from what that pane set, not the client's.
    palette: Box<[Option<u32>; 256]>,
    /// The rest of the OSC-set pane state — colours, path, progress — stored
    /// beside the palette for the same reason: the pane's own queries are
    /// answered from what the pane set, at the point in the stream it asked.
    osc_state: OscState,
}

impl Default for Observer {
    fn default() -> Self {
        Self {
            parser: Parser::default(),
            state: ObserverState {
                palette: Box::new([None; 256]),
                osc_state: OscState::default(),
            },
        }
    }
}

impl Observer {
    /// The bytes of a sequence the tokenizer has not finished, which
    /// `capture-pane -P` returns. The tokenizer's own accessor, which this is
    /// the public facade for.
    pub fn pending(&self) -> &[u8] {
        self.parser.pending()
    }

    /// Whether the tokenizer is waiting for a string terminator, which is when
    /// tmux's ground timer runs.
    pub fn awaiting_terminator(&self) -> bool {
        self.parser.awaiting_terminator()
    }

    /// The pane state OSC sequences have set, as the formats report it.
    pub fn osc_state(&self) -> OscState {
        self.state.osc_state.clone()
    }

    /// Abandon a sequence whose terminator never arrived. Nothing is observed,
    /// because tmux's ground timer discards the sequence rather than
    /// dispatching it.
    pub fn expire(&mut self) -> bool {
        self.parser.expire()
    }

    /// Tokenize one chunk of pane output against the current options.
    pub fn feed(&mut self, input: &[u8], policy: &OutputPolicy) -> Observed {
        self.parser.set_string_capacity(policy.input_buffer_size);
        // Observed where it is dispatched rather than after the chunk: the
        // tokenizer holds nothing a token's own handling reads, so staging the
        // chunk's tokens first would only move every one of them twice.
        let Self { parser, state } = self;
        let mut observed = Observed::default();
        parser.parse(input, |token| state.token(token, policy, &mut observed));
        observed
    }
}

impl Observer {
    /// Drop the pane's `OSC 4` palette, as tmux's `colour_palette_clear` does
    /// for a reset the pane's own byte stream did not carry.
    pub fn clear_palette(&mut self) {
        *self.state.palette = [None; 256];
    }
}

impl ObserverState {
    fn token(&mut self, token: Token, policy: &OutputPolicy, out: &mut Observed) {
        match &token.kind {
            TokenKind::Print(_) | TokenKind::Utf8Started | TokenKind::Replacement => {
                out.keep(token);
            }
            TokenKind::Control(byte) => {
                if *byte == 0x07 {
                    out.event(Event::Bell);
                }
                out.keep(token);
            }
            TokenKind::Esc {
                intermediates,
                final_byte,
            } => {
                // RIS returns the pane's own palette to the terminal's, the
                // way tmux's `INPUT_ESC_RIS` clears `wp->palette`.
                if intermediates.is_empty() && *final_byte == b'c' {
                    *self.palette = [None; 256];
                }
                out.keep(token)
            }
            TokenKind::Csi { .. } => self.csi(token, policy, out),
            TokenKind::Osc { .. } => self.osc(token, policy, out),
            TokenKind::Dcs {
                intermediates,
                final_byte,
                data,
            } => self.dcs(intermediates, *final_byte, data, policy, out),
            TokenKind::Apc { data } => {
                // tmux hands an APC title to the same `screen_set_title` as OSC
                // 0/2, and an emulator that does not recognize APC would print
                // it. Rewriting it as the OSC 2 it means keeps the two in
                // stream order, so the last one to arrive still wins.
                if policy.allow_set_title {
                    let mut body = b"2;".to_vec();
                    body.extend_from_slice(data);
                    out.rewrite(TokenKind::Osc {
                        data: body,
                        end: StringEnd::StringTerminator,
                    });
                }
            }
            TokenKind::Rename { data } => {
                // The screen/tmux rename control. Nothing downstream recognizes
                // it, so it is reported here and dropped from the stream rather
                // than printed as literal text.
                out.event(Event::Rename(String::from_utf8_lossy(data).into_owned()));
            }
        }
    }

    fn csi(&mut self, token: Token, policy: &OutputPolicy, out: &mut Observed) {
        let TokenKind::Csi {
            private,
            params,
            intermediates,
            final_byte,
        } = &token.kind
        else {
            return;
        };
        let (private, final_byte) = (*private, *final_byte);
        // tmux's `INPUT_CSI_MODSET`: with `extended-keys off` the pane's
        // `modifyOtherKeys` request is refused where it is parsed, so turning
        // the option on later does not revive it.
        if !policy.extended_keys
            && private == Some(b'>')
            && intermediates.is_empty()
            && matches!(final_byte, b'm' | b'n')
            && params
                .first()
                .is_none_or(|param| param.get(0) == Some(4) || param.get(4) == Some(4))
        {
            return;
        }
        match (private, intermediates.as_slice(), final_byte) {
            // DECSET / DECRST.
            (Some(b'?'), [], b'h' | b'l') => {
                return self.dec_mode(token, final_byte == b'h', policy, out);
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
            // DSR ?996: which theme is this terminal using?
            (Some(b'?'), [], b'n') if params.first().is_some_and(|p| p.get(0) == Some(996)) => {
                out.event(Event::ThemeQuery);
            }
            // Primary DA.
            (None, [], b'c') if first_is_zero(params) => {
                out.event(Event::Reply(b"\x1b[?1;2c".to_vec()));
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
            _ => {}
        }
        out.keep(token);
    }

    /// DECSET/DECRST: apply each parameter in turn, as `input.c` does, and drop
    /// the screen switches the `alternate-screen` option refuses.
    fn dec_mode(&mut self, token: Token, set: bool, policy: &OutputPolicy, out: &mut Observed) {
        let TokenKind::Csi { params, .. } = &token.kind else {
            return;
        };
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
        out.rewrite(TokenKind::Csi {
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
        });
    }

    fn osc(&mut self, token: Token, policy: &OutputPolicy, out: &mut Observed) {
        let TokenKind::Osc { data, end } = &token.kind else {
            return;
        };
        let end = *end;
        // tmux's `input_exit_osc`: a body that does not open with digits names
        // no command at all.
        let digits = data.iter().take_while(|byte| byte.is_ascii_digit()).count();
        if digits == 0 {
            out.keep(token);
            return;
        }
        // `input_exit_osc` accumulates into a `u_int`, so a run of digits too
        // long for one wraps rather than failing — and a pane can reach OSC 0 by
        // writing `8` and thirty zeros. The arithmetic is the observable part,
        // so it is reproduced rather than corrected.
        let number = data[..digits].iter().fold(0u32, |number, digit| {
            number
                .wrapping_mul(10)
                .wrapping_add(u32::from(digit - b'0'))
        });
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
            // The title is the screen's to keep; what the option decides is
            // whether the sequence reaches it at all.
            0 | 2 => {
                if policy.allow_set_title {
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
                    self.osc_state.path = path;
                }
            }
            9 => {
                if let Some((state, progress)) = progress_bar_report(&text) {
                    self.osc_state.progress_state = state;
                    if let Some(progress) = progress {
                        self.osc_state.progress_value = progress;
                    }
                }
            }
            10..=12 => {
                let stored = match number {
                    10 => &mut self.osc_state.foreground,
                    11 => &mut self.osc_state.background,
                    _ => &mut self.osc_state.cursor_colour,
                };
                if text == "?" {
                    match colour_query_reply(number, stored, end) {
                        Some(reply) => out.event(Event::Reply(reply)),
                        // With no colour of its own, tmux answers a background
                        // question from the attached terminal's; ask it. An
                        // unset foreground or cursor colour has no such
                        // fallback, so the question goes unanswered as tmux
                        // leaves it when no client offers one.
                        None if number == 11 => {
                            out.event(Event::ForwardQuery(BACKGROUND_COLOR_QUERY));
                        }
                        None => {}
                    }
                } else if let Some(colour) = parse_colour(&text) {
                    *stored = colour;
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
            110 => self.osc_state.foreground = "default".to_string(),
            111 => self.osc_state.background = "default".to_string(),
            112 => self.osc_state.cursor_colour = "none".to_string(),
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

    /// Hand the screen a token the observer built rather than one the pane
    /// wrote, which is how an option that refuses part of a sequence still lets
    /// the rest of it apply.
    fn rewrite(&mut self, kind: TokenKind) {
        self.screen.push(Token { kind });
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
            // Push and pop the title, one argument saying which title. The
            // stack itself is the screen's; the argument is still consumed so
            // the walk stays aligned, and a missing one abandons the rest of
            // the sequence, as `case -1` does.
            22 | 23 => {
                index += 1;
                if read(index).is_none() {
                    return;
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
fn progress_bar_report(body: &str) -> Option<(u8, Option<u8>)> {
    let rest = body.strip_prefix('4')?;
    // `9;4` and `9;4;` carry no state and are dropped rather than reset.
    let rest = rest.strip_prefix(';').filter(|rest| !rest.is_empty())?;
    let (state, rest) = rest.split_at_checked(1)?;
    let state = state.parse::<u8>().ok().filter(|state| *state <= 4)?;
    let Some(progress) = rest.strip_prefix(';').filter(|rest| !rest.is_empty()) else {
        // Anything other than a clean end here is malformed, not a bare state.
        return rest.is_empty().then_some((state, None));
    };
    let progress = progress.parse::<u8>().ok().filter(|value| *value <= 100)?;
    Some((state, Some(progress)))
}

/// The reply to an OSC 10/11/12 colour question, in the pane's own
/// terminator: tmux's `input_osc_colour_reply`. `None` when the stored value
/// is not a colour a reply can spell — the unset `default`/`none` forms.
fn colour_query_reply(number: u32, colour: &str, end: StringEnd) -> Option<Vec<u8>> {
    let packed = parse_packed_colour(colour)?;
    let (r, g, b) = ((packed >> 16) as u8, (packed >> 8) as u8, packed as u8);
    let terminator = match end {
        StringEnd::StringTerminator => "\x1b\\",
        StringEnd::Bell => "\x07",
    };
    Some(
        format!("\x1b]{number};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}{terminator}")
            .into_bytes(),
    )
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
pub fn parse_packed_colour(value: &str) -> Option<u32> {
    let text = parse_colour(value)?;
    u32::from_str_radix(text.strip_prefix('#')?, 16).ok()
}

/// An X11 colour specification as the `#rrggbb` string the formats report.
fn parse_colour(value: &str) -> Option<String> {
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

    /// The byte stream the screen tokens stand for, serialized back from what
    /// each token says rather than from bytes carried alongside it: what an
    /// emulator that parses for itself would have to be handed.
    fn screen_bytes(observed: &Observed) -> Vec<u8> {
        let mut out = Vec::new();
        for token in &observed.screen {
            match &token.kind {
                TokenKind::Print(character) => {
                    let mut buffer = [0u8; 4];
                    out.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                }
                // The bytes it has collected travel with the token it resolves
                // into, so an unfinished character contributes none of its own.
                TokenKind::Utf8Started => {}
                TokenKind::Replacement => out.extend_from_slice("\u{fffd}".as_bytes()),
                TokenKind::Control(byte) => out.push(*byte),
                TokenKind::Esc {
                    intermediates,
                    final_byte,
                } => {
                    out.push(0x1b);
                    out.extend_from_slice(intermediates);
                    out.push(*final_byte);
                }
                TokenKind::Csi {
                    private,
                    params,
                    intermediates,
                    final_byte,
                } => {
                    out.extend_from_slice(b"\x1b[");
                    out.extend(private);
                    let positions: Vec<String> = params
                        .iter()
                        .map(|param| {
                            let mut position = match param.value {
                                Some(value) => value.to_string(),
                                None => String::new(),
                            };
                            for sub in &param.subs {
                                position.push(':');
                                if let Some(sub) = sub {
                                    position.push_str(&sub.to_string());
                                }
                            }
                            position
                        })
                        .collect();
                    out.extend_from_slice(positions.join(";").as_bytes());
                    out.extend_from_slice(intermediates);
                    out.push(*final_byte);
                }
                TokenKind::Dcs {
                    intermediates,
                    final_byte,
                    data,
                } => {
                    out.extend_from_slice(b"\x1bP");
                    out.extend_from_slice(intermediates);
                    out.push(*final_byte);
                    out.extend_from_slice(data);
                    out.extend_from_slice(b"\x1b\\");
                }
                TokenKind::Osc { data, end } => {
                    out.extend_from_slice(b"\x1b]");
                    out.extend_from_slice(data);
                    match end {
                        StringEnd::StringTerminator => out.extend_from_slice(b"\x1b\\"),
                        StringEnd::Bell => out.push(0x07),
                    }
                }
                TokenKind::Apc { data } => {
                    out.extend_from_slice(b"\x1b_");
                    out.extend_from_slice(data);
                    out.extend_from_slice(b"\x1b\\");
                }
                TokenKind::Rename { data } => {
                    out.extend_from_slice(b"\x1bk");
                    out.extend_from_slice(data);
                    out.extend_from_slice(b"\x1b\\");
                }
            }
        }
        out
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

    /// The `ESC \\` stays: it ends the rename by leaving the state, and what
    /// follows the string is an escape of its own — the no-op ST — which the
    /// screen sees as `input_esc_dispatch` sees it.
    #[test]
    fn the_screen_rename_control_is_reported_and_removed() {
        let (_, observed) = observe(b"a\x1bkname\x1b\\b");
        assert_eq!(screen_bytes(&observed), b"a\x1b\\b");
        assert_eq!(
            observed.events,
            vec![(1, Event::Rename("name".to_string()))]
        );
    }

    #[test]
    fn an_apc_title_reaches_the_screen_as_the_osc_2_it_means() {
        let (_, observed) = observe(b"\x1b_title\x1b\\");
        assert_eq!(screen_bytes(&observed), b"\x1b]2;title\x1b\\\x1b\\");
        assert!(observed.events.is_empty(), "the title is the screen's");
    }

    #[test]
    fn a_refused_title_never_reaches_the_screen() {
        let mut observer = Observer::default();
        let refused = OutputPolicy {
            allow_set_title: false,
            ..policy()
        };
        let observed = observer.feed(b"x\x1b]2;t\x07\x1b_a\x1b\\y", &refused);
        assert_eq!(screen_bytes(&observed), b"x\x1b\\y");
        assert!(observed.events.is_empty());
    }

    /// `input_exit_osc` accumulates the option into a `u_int`, so a digit run
    /// too long for one wraps instead of being refused: `8` and thirty zeros is
    /// zero again, which is OSC 0 with an empty title — gated as a title, so
    /// the sequence is forwarded to the screen rather than reported.
    #[test]
    fn an_osc_option_wraps_the_way_a_u_int_does() {
        let (_, observed) = observe(b"\x1b]800000000000000000000000000000\x07");
        assert!(observed.events.is_empty());
        assert_eq!(observed.screen.len(), 1, "the wrapped OSC 0 is kept");
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
        assert!(
            observed
                .events
                .iter()
                .all(|(_, event)| !matches!(event, Event::AlternateScreen(_)))
        );
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
            vec![Event::Reply(b"\x1b[?1;2c".to_vec())]
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
        // XTSMGRAPHICS goes unanswered: tmux's `input_csi_dispatch_sm_graphics`
        // is an empty function in a build without sixel support.
        assert_eq!(events(b"\x1b[?1;1S"), vec![]);
    }

    #[test]
    fn a_colour_question_is_answered_from_what_the_pane_set() {
        let mut observer = Observer::default();
        // Unset: the foreground and cursor colour go unanswered; only the
        // background falls back to asking the outer terminal.
        let observed = observer.feed(b"\x1b]10;?\x07\x1b]12;?\x07\x1b]11;?\x07", &policy());
        assert_eq!(
            observed.events,
            vec![(2, Event::ForwardQuery(BACKGROUND_COLOR_QUERY))]
        );
        // Set: the stored colour answers, in the query's own terminator.
        let observed = observer.feed(b"\x1b]10;rgb:ff/00/00\x1b\\\x1b]10;?\x07", &policy());
        assert_eq!(
            observed
                .events
                .iter()
                .map(|(_, event)| event.clone())
                .collect::<Vec<_>>(),
            vec![Event::Reply(b"\x1b]10;rgb:ffff/0000/0000\x07".to_vec())]
        );
    }

    #[test]
    fn osc_colours_and_paths_are_stored_for_the_formats() {
        let mut observer = Observer::default();
        observer.feed(
            b"\x1b]11;rgb:ff/00/00\x1b\\\x1b]7;file://h/tmp\x1b\\\x1b]9;4;1;42\x07",
            &policy(),
        );
        let state = observer.osc_state();
        assert_eq!(state.background, "#ff0000");
        assert_eq!(state.path, "file://h/tmp");
        assert_eq!((state.progress_state, state.progress_value), (1, 42));
        // The reset forms put the unset spellings back.
        observer.feed(b"\x1b]111\x07\x1b]112\x07", &policy());
        let state = observer.osc_state();
        assert_eq!(state.background, "default");
        assert_eq!(state.cursor_colour, "none");
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
    fn decrqss_is_forwarded_unparsed_for_the_terminal_to_answer() {
        // The cursor style and the blink that answer this live on the screen,
        // which the observer does not hold; the terminal resolves the request
        // against it at this point in the stream.
        assert_eq!(
            events(b"\x1b[4 q\x1bP$q q\x1b\\"),
            vec![Event::StatusReport(b" q".to_vec())]
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
    fn tab_stop_edits_pass_through_as_screen_tokens() {
        // HTS and TBC are the screen's to apply — the engine owns the tab
        // stops — so the observer forwards them without an event.
        let (_, observed) = observe(b"\x1b[5G\x1bH\x1b[g\x1b[3g");
        assert!(observed.events.is_empty());
        assert_eq!(observed.screen.len(), 4);
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

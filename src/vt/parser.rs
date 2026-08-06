//! The pane tokenizer: one DEC ANSI state machine over a pane's byte stream.
//!
//! This is the Paul Williams parser as tmux's `input.c` implements it, with the
//! same deliberate departures: 7-bit only, UTF-8 collected in ground state, OSC
//! terminated by BEL as well as ST, an APC state, and a state for the
//! `ESC k … ST` window rename that screen-family terminfo entries emit.
//!
//! The parser is deliberately free of terminal semantics. It turns bytes into
//! [`Token`]s and nothing else; deciding what a token *means* — what it writes
//! to the grid, what it reports back to the pane, what server state it changes —
//! belongs to the layer above (see [`super::observer`]). That split is the point
//! of the tokenizer: every consumer that used to recover escape-sequence framing
//! on its own now shares this one framing.
//!
//! Each token carries the exact input bytes it was built from ([`Token::raw`]),
//! accumulated across reads the way tmux accumulates `since_ground`. A consumer
//! that has to hand a sequence on to another emulator can forward those bytes
//! verbatim rather than re-serializing parameters it may not round-trip.

/// Where the state machine is between bytes. The names follow `input.c`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum State {
    #[default]
    Ground,
    EscEnter,
    EscIntermediate,
    CsiEnter,
    CsiParameter,
    CsiIntermediate,
    CsiIgnore,
    DcsEnter,
    DcsParameter,
    DcsIntermediate,
    DcsHandler,
    DcsEscape,
    DcsIgnore,
    OscString,
    ApcString,
    RenameString,
    /// An `ESC` arrived inside a string. A following `\` is the ST that ends
    /// it; anything else means the ESC was the start of something new and the
    /// string ends where it stood.
    StringEscape,
    ConsumeSt,
}

/// Which string sequence [`State::StringEscape`] is suspended inside.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StringKind {
    Osc,
    Apc,
    Rename,
}

/// How a string-carrying sequence ended. tmux answers a query with the same
/// terminator the query used, so this has to survive to the dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StringEnd {
    /// `ESC \` (ST). tmux is 7-bit only, so the 8-bit `0x9c` is payload, not a
    /// terminator; leaving the string state is what dispatches it.
    StringTerminator,
    /// A bare `BEL`, which tmux accepts for OSC.
    Bell,
}

/// tmux's `param_buf` after splitting: one CSI/DCS parameter position, which
/// may be empty (defaulted) and may carry sub-parameters after a colon.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Param {
    /// The parameter proper, or `None` when the position was left empty.
    pub(crate) value: Option<u32>,
    /// Colon-separated sub-parameters, as SGR 38:2:… uses.
    pub(crate) subs: Vec<Option<u32>>,
}

impl Param {
    /// The parameter, or `default` when the position was empty — the reading
    /// every CSI handler in `input.c` performs through `input_get`.
    pub(crate) fn or(&self, default: u32) -> u32 {
        self.value.unwrap_or(default)
    }
}

/// One complete unit of a pane's byte stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    /// A printable character in ground state. Invalid UTF-8 has already been
    /// resolved to `U+FFFD`, one replacement per malformed sequence.
    Print(char),
    /// A C0 control executed in place (`BEL`, `BS`, `HT`, `LF`, `CR`, `SO`, …).
    Control(u8),
    /// `ESC` + intermediates + a final byte.
    Esc {
        intermediates: Vec<u8>,
        final_byte: u8,
    },
    /// `CSI` + an optional private marker + parameters + intermediates + final.
    Csi {
        /// The `<`…`?` byte when the sequence has one, as DEC private modes do.
        private: Option<u8>,
        params: Vec<Param>,
        intermediates: Vec<u8>,
        final_byte: u8,
    },
    /// `DCS` + parameters + intermediates + final + payload, ended by ST.
    Dcs {
        params: Vec<Param>,
        intermediates: Vec<u8>,
        final_byte: u8,
        data: Vec<u8>,
    },
    /// `OSC` + payload, ended by ST or BEL.
    Osc { data: Vec<u8>, end: StringEnd },
    /// `APC` + payload. Some terminals set the title with this.
    Apc { data: Vec<u8> },
    /// `ESC k` + payload: the screen/tmux window rename control.
    Rename { data: Vec<u8> },
}

/// A token plus the input bytes it was assembled from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    /// Every byte the parser consumed for this token, introducer and terminator
    /// included, in stream order.
    pub(crate) raw: Vec<u8>,
}

/// tmux's `INPUT_BUF_START`: the string buffer starts here and doubles.
const INPUT_BUFFER_START: usize = 32;

/// tmux's `INPUT_BUF_DEFAULT_SIZE`, the `input-buffer-size` default.
pub(crate) const INPUT_BUFFER_DEFAULT_SIZE: u32 = 1_048_576;

/// How long a terminal string may actually grow under `input-buffer-size`.
///
/// tmux's `input_input` doubles the buffer from [`INPUT_BUFFER_START`] and
/// abandons the string once the next doubling would pass the option, so the
/// usable capacity is the largest such power of two that fits — and a string is
/// discarded as soon as it would fill it.
fn input_buffer_capacity(limit: u32) -> usize {
    let limit = limit as usize;
    let mut capacity = INPUT_BUFFER_START;
    while capacity.saturating_mul(2) <= limit {
        capacity *= 2;
    }
    capacity
}

/// Tokenize a self-contained byte string in one go.
///
/// For a caller that has to *synthesize* a sequence — a reset the server
/// applies on the pane's behalf, a harness feeding a screen model — rather than
/// one reading a pane, where the parser state has to survive read boundaries.
pub(crate) fn tokenize(input: &[u8]) -> Vec<Token> {
    let mut parser = Parser::default();
    let mut tokens = Vec::new();
    parser.parse(input, |token| tokens.push(token));
    tokens
}

/// tmux's `param_buf` is a fixed 64 bytes and a sequence that overruns it is
/// abandoned.
const PARAM_BUFFER_LIMIT: usize = 64;

/// tmux's `interm_buf` is four bytes, likewise.
const INTERMEDIATE_BUFFER_LIMIT: usize = 4;

/// The DEC ANSI state machine over one pane's output.
pub(crate) struct Parser {
    state: State,
    /// Input bytes belonging to the token being assembled.
    raw: Vec<u8>,
    param: Vec<u8>,
    intermediate: Vec<u8>,
    /// The collected string payload of an OSC/DCS/APC/rename sequence.
    string: Vec<u8>,
    /// Set when the payload outgrew [`Self::string_capacity`]; the sequence is
    /// then dispatched as if it had never been collected, which is how tmux
    /// discards a runaway string.
    string_overflow: bool,
    /// `input-buffer-size`, as the option currently reads.
    string_capacity: usize,
    /// A UTF-8 sequence part-way through ground state.
    utf8: Utf8Collector,
    /// Which string [`State::StringEscape`] is suspended inside.
    string_kind: StringKind,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            state: State::default(),
            raw: Vec::new(),
            param: Vec::new(),
            intermediate: Vec::new(),
            string: Vec::new(),
            string_overflow: false,
            string_capacity: input_buffer_capacity(INPUT_BUFFER_DEFAULT_SIZE),
            utf8: Utf8Collector::default(),
            string_kind: StringKind::Osc,
        }
    }
}

impl Parser {
    /// Apply the current `input-buffer-size`. tmux reads the option each time
    /// it grows a string, so a change takes effect on the next sequence.
    pub(crate) fn set_string_capacity(&mut self, limit: u32) {
        self.string_capacity = input_buffer_capacity(limit);
    }

    /// Feed one chunk of pane output, calling `emit` once per complete token.
    ///
    /// A sequence split across chunks is retained, so the caller may hand over
    /// arbitrary read boundaries.
    pub(crate) fn parse(&mut self, input: &[u8], mut emit: impl FnMut(Token)) {
        for &byte in input {
            self.step(byte, &mut emit);
        }
    }

    fn step(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        // `anywhere` transitions: CAN and SUB abort into ground, ESC restarts.
        // The DCS payload states opt out, which is what lets a passthrough
        // payload carry arbitrary bytes.
        let anywhere = !matches!(
            self.state,
            State::DcsHandler | State::DcsEscape | State::StringEscape
        );
        if anywhere {
            match byte {
                0x18 | 0x1a => {
                    self.leave_state(emit);
                    self.raw.push(byte);
                    self.dispatch(TokenKind::Control(byte), emit);
                    return;
                }
                0x1b => {
                    if let Some(kind) = self.string_state() {
                        // Hold the ESC: it may be the ST that ends the string,
                        // in which case both bytes belong to that sequence.
                        self.string_kind = kind;
                        self.raw.push(byte);
                        self.state = State::StringEscape;
                        return;
                    }
                    self.leave_state(emit);
                    self.raw.push(byte);
                    self.state = State::EscEnter;
                    return;
                }
                _ => {}
            }
        }

        match self.state {
            State::Ground => self.ground(byte, emit),
            State::EscEnter => self.esc_enter(byte, emit),
            State::EscIntermediate => self.esc_intermediate(byte, emit),
            State::CsiEnter => self.csi_enter(byte, emit),
            State::CsiParameter => self.csi_parameter(byte, emit),
            State::CsiIntermediate => self.csi_intermediate(byte, emit),
            State::CsiIgnore => self.csi_ignore(byte, emit),
            State::DcsEnter => self.dcs_enter(byte, emit),
            State::DcsParameter => self.dcs_parameter(byte, emit),
            State::DcsIntermediate => self.dcs_intermediate(byte, emit),
            State::DcsHandler => self.dcs_handler(byte, emit),
            State::DcsEscape => self.dcs_escape(byte, emit),
            State::DcsIgnore => self.dcs_ignore(byte),
            State::OscString => self.osc_string(byte, emit),
            State::ApcString => self.apc_string(byte, emit),
            State::RenameString => self.rename_string(byte, emit),
            State::StringEscape => self.string_escape(byte, emit),
            State::ConsumeSt => self.consume_st(byte),
        }
    }

    /// Leave the current state because an `anywhere` transition fired.
    ///
    /// This is `input.c`'s `input_set_state`, which runs the old state's exit
    /// handler before entering the new one. OSC, APC and the screen rename are
    /// the states that *have* an exit handler, which is why `ESC \\` ends them
    /// even though no table entry matches ST: the ESC leaves the state, and
    /// leaving is what dispatches the string.
    ///
    /// A partly-collected UTF-8 character is resolved here too: the bytes seen
    /// so far can no longer complete, so they become one replacement character
    /// rather than being held across the interruption.
    fn leave_state(&mut self, emit: &mut impl FnMut(Token)) {
        match self.state {
            State::OscString => self.finish_string(
                |data| TokenKind::Osc {
                    data,
                    end: StringEnd::StringTerminator,
                },
                emit,
            ),
            State::ApcString => self.finish_string(|data| TokenKind::Apc { data }, emit),
            State::RenameString => self.finish_string(|data| TokenKind::Rename { data }, emit),
            State::Ground => {
                if let Some(character) = self.utf8.flush() {
                    let raw = std::mem::take(&mut self.raw);
                    emit(Token {
                        kind: TokenKind::Print(character),
                        raw,
                    });
                }
            }
            _ => {}
        }
        self.raw.clear();
        self.param.clear();
        self.intermediate.clear();
        self.string.clear();
        self.string_overflow = false;
        self.utf8.reset();
        self.state = State::Ground;
    }

    /// Emit a token and start the next one.
    fn dispatch(&mut self, kind: TokenKind, emit: &mut impl FnMut(Token)) {
        let raw = std::mem::take(&mut self.raw);
        self.param.clear();
        self.intermediate.clear();
        self.string.clear();
        self.string_overflow = false;
        self.state = State::Ground;
        emit(Token { kind, raw });
    }

    fn ground(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        // A high byte continues or opens a UTF-8 character; anything else ends
        // one that was still open.
        if byte >= 0x80 {
            self.raw.push(byte);
            match self.utf8.push(byte) {
                Utf8Step::More => {}
                Utf8Step::Done(character) => {
                    let trailing = self.utf8.take_trailing();
                    self.dispatch(TokenKind::Print(character), emit);
                    // A malformed sequence can be terminated by a byte that
                    // itself opens the next one; replay it rather than lose it.
                    for byte in trailing {
                        self.ground(byte, emit);
                    }
                }
            }
            return;
        }
        if let Some(character) = self.utf8.flush() {
            let raw = std::mem::take(&mut self.raw);
            emit(Token {
                kind: TokenKind::Print(character),
                raw,
            });
        }
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x7e => {
                let kind = TokenKind::Print(byte as char);
                self.dispatch(kind, emit);
            }
            // DEL is a control in tmux, printed by nothing.
            _ => {
                self.raw.clear();
            }
        }
    }

    fn esc_enter(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x2f => {
                self.push_intermediate(byte);
                self.state = State::EscIntermediate;
            }
            0x50 => {
                self.state = State::DcsEnter;
            }
            0x58 | 0x5e => self.state = State::ConsumeSt,
            0x5b => self.state = State::CsiEnter,
            0x5d => self.state = State::OscString,
            0x5f => self.state = State::ApcString,
            0x6b => self.state = State::RenameString,
            0x30..=0x7e => {
                let intermediates = std::mem::take(&mut self.intermediate);
                self.dispatch(
                    TokenKind::Esc {
                        intermediates,
                        final_byte: byte,
                    },
                    emit,
                );
            }
            // 0x7f and the high bytes are ignored inside an escape.
            _ => {}
        }
    }

    fn esc_intermediate(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x2f => self.push_intermediate(byte),
            0x30..=0x7e => {
                let intermediates = std::mem::take(&mut self.intermediate);
                self.dispatch(
                    TokenKind::Esc {
                        intermediates,
                        final_byte: byte,
                    },
                    emit,
                );
            }
            _ => {}
        }
    }

    fn csi_enter(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x2f => {
                self.push_intermediate(byte);
                self.state = State::CsiIntermediate;
            }
            0x30..=0x3b => {
                self.push_param(byte);
                self.state = State::CsiParameter;
            }
            0x3c..=0x3f => {
                self.push_intermediate(byte);
                self.state = State::CsiParameter;
            }
            0x40..=0x7e => self.csi_dispatch(byte, emit),
            _ => {}
        }
    }

    fn csi_parameter(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x2f => {
                self.push_intermediate(byte);
                self.state = State::CsiIntermediate;
            }
            0x30..=0x3b => self.push_param(byte),
            0x3c..=0x3f => self.state = State::CsiIgnore,
            0x40..=0x7e => self.csi_dispatch(byte, emit),
            _ => {}
        }
    }

    fn csi_intermediate(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            0x20..=0x2f => self.push_intermediate(byte),
            0x30..=0x3f => self.state = State::CsiIgnore,
            0x40..=0x7e => self.csi_dispatch(byte, emit),
            _ => {}
        }
    }

    fn csi_ignore(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.dispatch(TokenKind::Control(byte), emit),
            // The sequence is abandoned: the final byte ends it and nothing is
            // reported, but the bytes still belong to it.
            0x40..=0x7e => {
                self.raw.clear();
                self.param.clear();
                self.intermediate.clear();
                self.state = State::Ground;
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, final_byte: u8, emit: &mut impl FnMut(Token)) {
        // A parameter run that overran `param_buf` is not reported at all.
        if self.param.len() > PARAM_BUFFER_LIMIT
            || self.intermediate.len() > INTERMEDIATE_BUFFER_LIMIT
        {
            self.raw.clear();
            self.param.clear();
            self.intermediate.clear();
            self.state = State::Ground;
            return;
        }
        // The private marker is an intermediate in `0x3c..=0x3f` collected
        // before the parameters; anything else stays an ordinary intermediate.
        let private = self
            .intermediate
            .first()
            .copied()
            .filter(|byte| (0x3c..=0x3f).contains(byte));
        let intermediates = if private.is_some() {
            self.intermediate[1..].to_vec()
        } else {
            std::mem::take(&mut self.intermediate)
        };
        let params = split_params(&self.param);
        self.dispatch(
            TokenKind::Csi {
                private,
                params,
                intermediates,
                final_byte,
            },
            emit,
        );
    }

    fn dcs_enter(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => {}
            0x20..=0x2f => {
                self.push_intermediate(byte);
                self.state = State::DcsIntermediate;
            }
            0x30..=0x39 | 0x3b => {
                self.push_param(byte);
                self.state = State::DcsParameter;
            }
            0x3a => self.state = State::DcsIgnore,
            0x3c..=0x3f => {
                self.push_intermediate(byte);
                self.state = State::DcsParameter;
            }
            0x40..=0x7e => self.dcs_start(byte),
            _ => {}
        }
        let _ = emit;
    }

    fn dcs_parameter(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => {}
            0x20..=0x2f => {
                self.push_intermediate(byte);
                self.state = State::DcsIntermediate;
            }
            0x30..=0x39 | 0x3b => self.push_param(byte),
            0x3a | 0x3c..=0x3f => self.state = State::DcsIgnore,
            0x40..=0x7e => self.dcs_start(byte),
            _ => {}
        }
        let _ = emit;
    }

    fn dcs_intermediate(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => {}
            0x20..=0x2f => self.push_intermediate(byte),
            0x30..=0x3f => self.state = State::DcsIgnore,
            0x40..=0x7e => self.dcs_start(byte),
            _ => {}
        }
        let _ = emit;
    }

    /// The DCS final byte arrives: the payload starts here.
    fn dcs_start(&mut self, final_byte: u8) {
        self.string.clear();
        self.string_overflow = false;
        // The final byte is the first byte of tmux's DCS input string.
        self.push_string(final_byte);
        self.state = State::DcsHandler;
    }

    fn dcs_handler(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        if byte == 0x1b {
            self.state = State::DcsEscape;
            return;
        }
        self.push_string(byte);
        let _ = emit;
    }

    fn dcs_escape(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        if byte == b'\\' {
            self.finish_dcs(emit);
            return;
        }
        // Not a terminator: the ESC was payload, and so is this byte. Doubling
        // an ESC is how a passthrough payload carries one.
        self.push_string(byte);
        self.state = State::DcsHandler;
    }

    fn finish_dcs(&mut self, emit: &mut impl FnMut(Token)) {
        if self.string_overflow {
            self.raw.clear();
            self.param.clear();
            self.intermediate.clear();
            self.string.clear();
            self.string_overflow = false;
            self.state = State::Ground;
            return;
        }
        let mut data = std::mem::take(&mut self.string);
        // The first collected byte is the sequence's final byte, not payload.
        let final_byte = if data.is_empty() { 0 } else { data.remove(0) };
        let params = split_params(&self.param);
        let intermediates = std::mem::take(&mut self.intermediate);
        self.dispatch(
            TokenKind::Dcs {
                params,
                intermediates,
                final_byte,
                data,
            },
            emit,
        );
    }

    fn dcs_ignore(&mut self, byte: u8) {
        self.raw.push(byte);
    }

    fn osc_string(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x07 => self.finish_string(
                |data| TokenKind::Osc {
                    data,
                    end: StringEnd::Bell,
                },
                emit,
            ),
            0x20..=0xff => self.push_string(byte),
            _ => {}
        }
    }

    fn apc_string(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            // Divergence: tmux ignores a BEL inside APC and keeps collecting
            // until an ESC. hmux ends the string, because the terminfo `fsl`
            // of several screen-family entries is a bare BEL and swallowing
            // everything after it loses real output.
            0x07 => self.finish_string(|data| TokenKind::Apc { data }, emit),
            0x20..=0xff => self.push_string(byte),
            _ => {}
        }
    }

    fn rename_string(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            // Same divergence as [`Self::apc_string`]: a BEL ends the rename.
            0x07 => self.finish_string(|data| TokenKind::Rename { data }, emit),
            0x20..=0xff => self.push_string(byte),
            _ => {}
        }
    }

    fn consume_st(&mut self, byte: u8) {
        self.raw.push(byte);
    }

    /// Which string sequence the parser is collecting, if any.
    fn string_state(&self) -> Option<StringKind> {
        match self.state {
            State::OscString => Some(StringKind::Osc),
            State::ApcString => Some(StringKind::Apc),
            State::RenameString => Some(StringKind::Rename),
            _ => None,
        }
    }

    /// Dispatch the suspended string. `terminated` says whether the held `ESC`
    /// turned out to be the ST that ends it, and so belongs to its bytes.
    fn finish_suspended_string(&mut self, terminated: bool, emit: &mut impl FnMut(Token)) {
        if !terminated {
            // The ESC starts something else; it is not part of this sequence.
            self.raw.pop();
        }
        let kind = self.string_kind;
        self.state = match kind {
            StringKind::Osc => State::OscString,
            StringKind::Apc => State::ApcString,
            StringKind::Rename => State::RenameString,
        };
        match kind {
            StringKind::Osc => self.finish_string(
                |data| TokenKind::Osc {
                    data,
                    end: StringEnd::StringTerminator,
                },
                emit,
            ),
            StringKind::Apc => self.finish_string(|data| TokenKind::Apc { data }, emit),
            StringKind::Rename => self.finish_string(|data| TokenKind::Rename { data }, emit),
        }
    }

    fn string_escape(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        if byte == b'\\' {
            self.raw.push(byte);
            self.finish_suspended_string(true, emit);
            return;
        }
        // Not an ST: end the string where it stood, then let the ESC and this
        // byte go through the machine as the new sequence they are.
        self.finish_suspended_string(false, emit);
        self.step(0x1b, emit);
        self.step(byte, emit);
    }

    /// Complete a string sequence, or drop it when the payload ran away.
    fn finish_string(
        &mut self,
        kind: impl FnOnce(Vec<u8>) -> TokenKind,
        emit: &mut impl FnMut(Token),
    ) {
        if self.string_overflow {
            self.raw.clear();
            self.string.clear();
            self.string_overflow = false;
            self.state = State::Ground;
            return;
        }
        let data = std::mem::take(&mut self.string);
        self.dispatch(kind(data), emit);
    }

    /// Collect one parameter byte. An overrun past `param_buf` is not clamped:
    /// the dispatch checks the length and drops the sequence whole, as tmux's
    /// `input_parameter` does.
    fn push_param(&mut self, byte: u8) {
        self.param.push(byte);
    }

    fn push_intermediate(&mut self, byte: u8) {
        if self.intermediate.len() <= INTERMEDIATE_BUFFER_LIMIT {
            self.intermediate.push(byte);
        }
    }

    fn push_string(&mut self, byte: u8) {
        if self.string.len() + 1 >= self.string_capacity {
            self.string_overflow = true;
            return;
        }
        self.string.push(byte);
    }
}

/// tmux splits `param_buf` on `;`, reads each position as a number, and treats
/// an empty position as "not given". A position may carry colon-separated
/// sub-parameters, which SGR 38/48 use for direct colours.
fn split_params(buffer: &[u8]) -> Vec<Param> {
    if buffer.is_empty() {
        return Vec::new();
    }
    buffer
        .split(|byte| *byte == b';')
        .map(|position| {
            let mut parts = position.split(|byte| *byte == b':');
            let value = parts.next().map(parse_number).unwrap_or(None);
            let subs = parts.map(parse_number).collect();
            Param { value, subs }
        })
        .collect()
}

fn parse_number(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() {
        return None;
    }
    let mut value: u32 = 0;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value
            .saturating_mul(10)
            .saturating_add(u32::from(byte - b'0'));
    }
    Some(value)
}

/// The UTF-8 sequence being collected in ground state.
///
/// One malformed sequence becomes one `U+FFFD`, not one per byte — the cell
/// count tmux ends up with. A byte that cannot continue the sequence but could
/// open a new one is handed back so the caller can replay it.
#[derive(Default)]
struct Utf8Collector {
    bytes: Vec<u8>,
    /// Expected total length of the sequence in progress.
    width: usize,
    /// Bytes that terminated a malformed sequence and must be reconsidered.
    trailing: Vec<u8>,
}

enum Utf8Step {
    /// The sequence needs more bytes.
    More,
    /// A character is ready. The caller must also drain [`Utf8Collector::take_trailing`].
    Done(char),
}

const REPLACEMENT: char = '\u{fffd}';

impl Utf8Collector {
    fn push(&mut self, byte: u8) -> Utf8Step {
        if self.bytes.is_empty() {
            self.width = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                // A continuation byte or an over-long lead opens nothing.
                _ => return Utf8Step::Done(REPLACEMENT),
            };
            self.bytes.push(byte);
            return Utf8Step::More;
        }
        if !(0x80..=0xbf).contains(&byte) {
            // The sequence is malformed; this byte may still open the next one.
            self.bytes.clear();
            self.trailing.push(byte);
            return Utf8Step::Done(REPLACEMENT);
        }
        self.bytes.push(byte);
        if self.bytes.len() < self.width {
            return Utf8Step::More;
        }
        let character = std::str::from_utf8(&self.bytes)
            .ok()
            .and_then(|text| text.chars().next())
            .unwrap_or(REPLACEMENT);
        self.bytes.clear();
        Utf8Step::Done(character)
    }

    fn take_trailing(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.trailing)
    }

    /// Give up on an incomplete sequence because a non-UTF-8 byte arrived.
    fn flush(&mut self) -> Option<char> {
        (!self.bytes.is_empty()).then(|| {
            self.bytes.clear();
            REPLACEMENT
        })
    }

    fn reset(&mut self) {
        self.bytes.clear();
        self.trailing.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &[u8]) -> Vec<TokenKind> {
        let mut parser = Parser::default();
        let mut out = Vec::new();
        parser.parse(input, |token| out.push(token.kind));
        out
    }

    fn raws(input: &[u8]) -> Vec<Vec<u8>> {
        let mut parser = Parser::default();
        let mut out = Vec::new();
        parser.parse(input, |token| out.push(token.raw));
        out
    }

    #[test]
    fn ground_text_and_controls_are_separate_tokens() {
        assert_eq!(
            tokens(b"ab\r\n"),
            vec![
                TokenKind::Print('a'),
                TokenKind::Print('b'),
                TokenKind::Control(b'\r'),
                TokenKind::Control(b'\n'),
            ]
        );
    }

    #[test]
    fn csi_carries_private_marker_parameters_and_final() {
        assert_eq!(
            tokens(b"\x1b[?1049h"),
            vec![TokenKind::Csi {
                private: Some(b'?'),
                params: vec![Param {
                    value: Some(1049),
                    subs: Vec::new()
                }],
                intermediates: Vec::new(),
                final_byte: b'h',
            }]
        );
    }

    #[test]
    fn csi_keeps_empty_positions_and_subparameters() {
        let TokenKind::Csi { params, .. } = &tokens(b"\x1b[38:2::1:2:3;;4m")[0] else {
            panic!("expected a CSI");
        };
        assert_eq!(params[0].value, Some(38));
        assert_eq!(
            params[0].subs,
            vec![Some(2), None, Some(1), Some(2), Some(3)]
        );
        assert_eq!(params[1].value, None);
        assert_eq!(params[2].value, Some(4));
    }

    #[test]
    fn csi_intermediate_reaches_the_dispatch() {
        assert_eq!(
            tokens(b"\x1b[2 q"),
            vec![TokenKind::Csi {
                private: None,
                params: vec![Param {
                    value: Some(2),
                    subs: Vec::new()
                }],
                intermediates: vec![b' '],
                final_byte: b'q',
            }]
        );
    }

    #[test]
    fn osc_accepts_both_terminators() {
        assert_eq!(
            tokens(b"\x1b]2;hi\x07"),
            vec![TokenKind::Osc {
                data: b"2;hi".to_vec(),
                end: StringEnd::Bell,
            }]
        );
        assert_eq!(
            tokens(b"\x1b]2;hi\x1b\\"),
            vec![TokenKind::Osc {
                data: b"2;hi".to_vec(),
                end: StringEnd::StringTerminator,
            }]
        );
    }

    #[test]
    fn dcs_payload_keeps_a_doubled_escape() {
        assert_eq!(
            tokens(b"\x1bPtmux;\x1b\x1b[31m\x1b\\"),
            vec![TokenKind::Dcs {
                params: Vec::new(),
                intermediates: Vec::new(),
                final_byte: b't',
                data: b"mux;\x1b[31m".to_vec(),
            }]
        );
    }

    #[test]
    fn decrqss_arrives_with_its_intermediate_and_payload() {
        assert_eq!(
            tokens(b"\x1bP$q q\x1b\\"),
            vec![TokenKind::Dcs {
                params: Vec::new(),
                intermediates: vec![b'$'],
                final_byte: b'q',
                data: b" q".to_vec(),
            }]
        );
    }

    #[test]
    fn rename_and_apc_strings_are_their_own_tokens() {
        assert_eq!(
            tokens(b"\x1bkname\x1b\\"),
            vec![TokenKind::Rename {
                data: b"name".to_vec()
            }]
        );
        assert_eq!(
            tokens(b"\x1b_title\x1b\\"),
            vec![TokenKind::Apc {
                data: b"title".to_vec()
            }]
        );
    }

    #[test]
    fn raw_bytes_reproduce_the_whole_sequence() {
        assert_eq!(
            raws(b"a\x1b[1;2H\x1b]0;t\x07"),
            vec![
                b"a".to_vec(),
                b"\x1b[1;2H".to_vec(),
                b"\x1b]0;t\x07".to_vec()
            ]
        );
    }

    #[test]
    fn a_sequence_split_across_reads_is_still_one_token() {
        let mut parser = Parser::default();
        let mut out = Vec::new();
        parser.parse(b"\x1b[?10", |token| out.push(token));
        assert!(out.is_empty());
        parser.parse(b"49h", |token| out.push(token));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].raw, b"\x1b[?1049h");
    }

    #[test]
    fn an_escape_abandons_the_sequence_in_progress() {
        // `input.c`'s anywhere transition: ESC restarts wherever it lands.
        assert_eq!(
            tokens(b"\x1b[12\x1b[H"),
            vec![TokenKind::Csi {
                private: None,
                params: Vec::new(),
                intermediates: Vec::new(),
                final_byte: b'H',
            }]
        );
    }

    #[test]
    fn utf8_is_collected_in_ground_state() {
        assert_eq!(
            tokens("é界".as_bytes()),
            vec![TokenKind::Print('é'), TokenKind::Print('界')]
        );
    }

    #[test]
    fn one_malformed_sequence_becomes_one_replacement() {
        assert_eq!(
            tokens(b"\xc3\x28"),
            vec![TokenKind::Print('\u{fffd}'), TokenKind::Print('(')]
        );
        assert_eq!(tokens(b"\xff"), vec![TokenKind::Print('\u{fffd}')]);
    }

    #[test]
    fn an_incomplete_utf8_character_waits_for_the_next_read() {
        let mut parser = Parser::default();
        let mut out = Vec::new();
        parser.parse("界".as_bytes()[..2].as_ref(), |token| out.push(token.kind));
        assert!(out.is_empty());
        parser.parse("界".as_bytes()[2..].as_ref(), |token| out.push(token.kind));
        assert_eq!(out, vec![TokenKind::Print('界')]);
    }

    #[test]
    fn del_is_a_control_and_prints_nothing() {
        assert_eq!(
            tokens(b"a\x7fb"),
            vec![TokenKind::Print('a'), TokenKind::Print('b')]
        );
    }

    #[test]
    fn a_runaway_string_is_discarded_whole() {
        let mut parser = Parser::default();
        parser.set_string_capacity(64);
        let mut input = b"\x1b]2;".to_vec();
        input.extend(std::iter::repeat_n(b'x', 200));
        input.extend_from_slice(b"\x07");
        let mut out = Vec::new();
        parser.parse(&input, |token| out.push(token.kind));
        assert!(out.is_empty(), "the sequence is dropped, got {out:?}");
    }

    #[test]
    fn csi_ignore_swallows_the_rest_of_the_sequence() {
        assert_eq!(tokens(b"\x1b[1;2<Hx"), vec![TokenKind::Print('x')]);
    }
}

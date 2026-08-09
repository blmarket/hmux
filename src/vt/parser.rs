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
    /// The reading every CSI handler in `input.c` performs through `input_get`:
    /// the parameter, `default` when the position was left empty, and `None` —
    /// tmux's -1 — when it carries sub-parameters.
    ///
    /// `input_split` types a position with a colon in it as `INPUT_STRING`
    /// rather than as a number, and a handler that reads one abandons its
    /// operation instead of acting on the part before the colon. Only SGR looks
    /// past that, through [`Param::subs`].
    pub(crate) fn get(&self, default: u32) -> Option<u32> {
        if self.is_string() {
            return None;
        }
        Some(self.value.unwrap_or(default))
    }

    /// Whether the position carried sub-parameters — `input_split`'s
    /// `INPUT_STRING`.
    pub(crate) fn is_string(&self) -> bool {
        !self.subs.is_empty()
    }
}

/// One complete unit of a pane's byte stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    /// A printable character in ground state.
    Print(char),
    /// A UTF-8 sequence has just opened in ground state and has not finished.
    ///
    /// Nothing reaches the screen, and the bytes stay with the parser until the
    /// character resolves. It exists because tmux clears `INPUT_LAST` on every
    /// top-bit-set byte, so an unfinished character is already enough to leave
    /// a following REP with nothing to repeat.
    Utf8Started,
    /// The `U+FFFD` a malformed UTF-8 sequence resolves to, one per sequence.
    ///
    /// It is written to the screen exactly as a printed character is, and is
    /// kept apart from one only because tmux leaves `INPUT_LAST` clear for it:
    /// `input_stop_utf8` writes the replacement without recording it, so a
    /// following REP has nothing to repeat.
    Replacement,
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
        /// What `input_split` made of the parameters, or `None` when it refused
        /// them. Only the sixel branch of `input_dcs_dispatch` splits at all,
        /// and a refusal stops that branch rather than the whole sequence — so
        /// the distinction has to survive as far as the handler.
        params: Option<Vec<Param>>,
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

    /// The bytes of a sequence the parser is still part-way through, which is
    /// what `capture-pane -P` returns.
    ///
    /// This is tmux's `input_pending`. tmux accumulates into `since_ground`
    /// only while the state machine is *out* of ground state and drains it on
    /// the way back in, so a partly-collected UTF-8 character is not pending
    /// input: `input.c` collects those without leaving ground, and so does
    /// this.
    pub(crate) fn pending(&self) -> &[u8] {
        if self.state == State::Ground {
            &[]
        } else {
            &self.raw
        }
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
                    // CAN and SUB reach `input_c0_dispatch`, which stops UTF-8
                    // collection before the transition takes the parser home.
                    self.stop_utf8(emit);
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
    /// A partly-collected UTF-8 character survives this. `input_stop_utf8` is
    /// reached from `input_print`, `input_c0_dispatch` and `input_top_bit_set`
    /// and from nowhere else, so an escape sequence that interrupts a UTF-8
    /// character does not resolve it — the replacement lands where the stream
    /// next reaches ground, carrying whatever cell the sequence left behind.
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
            _ => {}
        }
        self.raw.clear();
        self.param.clear();
        self.intermediate.clear();
        self.string.clear();
        self.string_overflow = false;
        self.state = State::Ground;
    }

    /// `input_stop_utf8`: give up on a part-collected UTF-8 character and enter
    /// the replacement it decodes to.
    fn stop_utf8(&mut self, emit: &mut impl FnMut(Token)) {
        if self.utf8.flush() {
            let raw = std::mem::take(&mut self.raw);
            emit(Token {
                kind: TokenKind::Replacement,
                raw,
            });
        }
    }

    /// Execute a C0 control that arrived part-way through a sequence.
    ///
    /// `input.c` reaches `input_c0_dispatch` from table entries whose target
    /// state is NULL, so the control runs where it stands and neither the state
    /// nor the collected parameters move: `CSI 2 BEL ; 3 H` rings the bell and
    /// then runs `CUP(2,3)`. The byte stays in the sequence's accumulated bytes,
    /// which is what tmux's `since_ground` holds.
    fn execute_c0(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        // `input_c0_dispatch` stops UTF-8 collection before it looks at the
        // byte, the same as `input_print` does. The bytes it resolves belong to
        // the sequence in progress, so they stay in its accumulated bytes.
        if self.utf8.flush() {
            emit(Token {
                kind: TokenKind::Replacement,
                raw: Vec::new(),
            });
        }
        emit(Token {
            kind: TokenKind::Control(byte),
            raw: vec![byte],
        });
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
            let opening = self.utf8.is_idle();
            let step = self.utf8.push(byte);
            if opening && matches!(step, Utf8Step::More) {
                emit(Token {
                    kind: TokenKind::Utf8Started,
                    raw: Vec::new(),
                });
            }
            match step {
                Utf8Step::More => {}
                Utf8Step::Done(character) => self.dispatch(TokenKind::Print(character), emit),
                Utf8Step::Invalid => self.dispatch(TokenKind::Replacement, emit),
            }
            return;
        }
        // DEL is the one byte tmux's ground table gives no handler at all, so
        // it reaches neither `input_print` nor `input_c0_dispatch` and a UTF-8
        // character part-way through survives it: `\xcd\x7f` writes nothing,
        // and a continuation byte after the DEL still completes the character.
        if byte != 0x7f {
            self.stop_utf8(emit);
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
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
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
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
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
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
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
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
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
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
            0x20..=0x2f => self.push_intermediate(byte),
            0x30..=0x3f => self.state = State::CsiIgnore,
            0x40..=0x7e => self.csi_dispatch(byte, emit),
            _ => {}
        }
    }

    fn csi_ignore(&mut self, byte: u8, emit: &mut impl FnMut(Token)) {
        self.raw.push(byte);
        match byte {
            0x00..=0x17 | 0x19 | 0x1c..=0x1f => self.execute_c0(byte, emit),
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
        // A parameter list `input_split` refuses drops the sequence too:
        // `input_csi_dispatch` splits before it looks a handler up, and returns
        // where the split fails.
        let Some(params) = split_params(&self.param) else {
            self.raw.clear();
            self.param.clear();
            self.intermediate.clear();
            self.state = State::Ground;
            return;
        };
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

/// tmux's `param_list` holds this many positions, and `input_split` gives up on
/// the sequence — not on the excess — once they are all used.
const PARAM_LIMIT: usize = 24;

/// The largest value `strtonum(out, 0, INT_MAX, &errstr)` converts. A position
/// that does not fit fails the split, and the sequence goes with it.
const PARAM_MAX: u32 = i32::MAX as u32;

/// tmux splits `param_buf` on `;`, reads each position as a number, and treats
/// an empty position as "not given". A position may carry colon-separated
/// sub-parameters, which SGR 38/48 use for direct colours.
///
/// `None` is `input_split` returning -1: the sequence is abandoned before any
/// handler sees it, which is what a position too large for an `int` and a
/// twenty-fifth position both do.
fn split_params(buffer: &[u8]) -> Option<Vec<Param>> {
    if buffer.is_empty() {
        return Some(Vec::new());
    }
    let mut params = Vec::new();
    for position in buffer.split(|byte| *byte == b';') {
        let param = if position.is_empty() {
            // INPUT_MISSING: the position was left for the handler's default.
            Param::default()
        } else if position.contains(&b':') {
            // INPUT_STRING: never read as a number, so its size is nobody's
            // business until SGR takes the sub-parameters apart.
            let mut parts = position.split(|byte| *byte == b':');
            let value = parts.next().and_then(parse_number);
            let subs = parts.map(parse_number).collect();
            Param { value, subs }
        } else {
            // INPUT_NUMBER, through a `strtonum` that will not take a value
            // wider than an `int`.
            let value = parse_number(position).filter(|value| *value <= PARAM_MAX)?;
            Param {
                value: Some(value),
                subs: Vec::new(),
            }
        };
        params.push(param);
        if params.len() == PARAM_LIMIT {
            return None;
        }
    }
    Some(params)
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

/// The UTF-8 sequence being collected in ground state, tmux's `utf8_open` and
/// `utf8_append`.
///
/// One malformed sequence becomes one `U+FFFD`, and the length that decides
/// where "one sequence" ends is the one the *lead byte* announced. A byte that
/// cannot continue the sequence does not end it: it poisons the result and is
/// swallowed with the rest, so a three-byte lead consumes three bytes whatever
/// they turn out to be. That is `utf8_append` setting `ud->width = 0xff` and
/// carrying on, and it is why a valid character buried inside a bad sequence
/// does not reappear — tmux never reconsiders those bytes.
#[derive(Default)]
struct Utf8Collector {
    bytes: Vec<u8>,
    /// Expected total length of the sequence in progress.
    width: usize,
    /// tmux's `ud->width = 0xff`: something arrived that cannot be part of this
    /// sequence, so whatever it collects decodes to one replacement.
    poisoned: bool,
}

enum Utf8Step {
    /// The sequence needs more bytes.
    More,
    /// A character is ready.
    Done(char),
    /// The sequence was malformed and stands for one replacement. This is not
    /// `Done(REPLACEMENT)`: a stream may spell `U+FFFD` correctly, and that is
    /// a printed character like any other.
    Invalid,
}

impl Utf8Collector {
    /// Whether no sequence is part-way through, so the next byte opens one.
    fn is_idle(&self) -> bool {
        self.bytes.is_empty()
    }

    fn push(&mut self, byte: u8) -> Utf8Step {
        if self.bytes.is_empty() {
            self.width = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                // A continuation byte or an over-long lead opens nothing, and
                // nothing is collected: `utf8_open` fails before it appends.
                _ => return Utf8Step::Invalid,
            };
            self.poisoned = false;
            self.bytes.push(byte);
            return Utf8Step::More;
        }
        // A byte that is not a continuation still belongs to this sequence —
        // it just guarantees the sequence is bad.
        self.poisoned |= !(0x80..=0xbf).contains(&byte);
        self.bytes.push(byte);
        if self.bytes.len() < self.width {
            return Utf8Step::More;
        }
        let character = if self.poisoned {
            None
        } else {
            std::str::from_utf8(&self.bytes)
                .ok()
                .and_then(|text| text.chars().next())
        };
        self.bytes.clear();
        match character {
            Some(character) => Utf8Step::Done(character),
            None => Utf8Step::Invalid,
        }
    }

    /// Give up on an incomplete sequence because a byte that cannot be part of
    /// one at all arrived — anything below `0x80`, which the ground state
    /// handles itself. tmux's `input_stop_utf8`, called from `input_print` and
    /// `input_c0_dispatch` before either looks at the byte.
    fn flush(&mut self) -> bool {
        let started = !self.bytes.is_empty();
        self.bytes.clear();
        started
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
                params: Some(Vec::new()),
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
                params: Some(Vec::new()),
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
            vec![
                TokenKind::Utf8Started,
                TokenKind::Print('é'),
                TokenKind::Utf8Started,
                TokenKind::Print('界')
            ]
        );
    }

    #[test]
    fn one_malformed_sequence_becomes_one_replacement() {
        assert_eq!(
            tokens(b"\xc3\x28"),
            vec![
                TokenKind::Utf8Started,
                TokenKind::Replacement,
                TokenKind::Print('(')
            ]
        );
        assert_eq!(tokens(b"\xff"), vec![TokenKind::Replacement]);
    }

    #[test]
    fn a_bad_sequence_swallows_the_length_its_lead_byte_announced() {
        // A three-byte lead consumes three bytes even when the second is a
        // perfectly good two-byte character: `utf8_append` poisons the result
        // and keeps collecting, so the buried character never reappears.
        assert_eq!(
            tokens(b"\xe0\xc3\xa9"),
            vec![TokenKind::Utf8Started, TokenKind::Replacement],
            "one sequence, one replacement, nothing replayed"
        );
        // The same sequence one byte shorter leaves the third slot to whatever
        // comes next, which is also swallowed.
        assert_eq!(
            tokens(b"\xe0\xc3\xa9\xc3\xa9"),
            vec![
                TokenKind::Utf8Started,
                TokenKind::Replacement,
                TokenKind::Utf8Started,
                TokenKind::Print('é')
            ]
        );
    }

    #[test]
    fn an_incomplete_utf8_character_waits_for_the_next_read() {
        let mut parser = Parser::default();
        let mut out = Vec::new();
        parser.parse("界".as_bytes()[..2].as_ref(), |token| out.push(token.kind));
        assert_eq!(out, vec![TokenKind::Utf8Started]);
        parser.parse("界".as_bytes()[2..].as_ref(), |token| out.push(token.kind));
        assert_eq!(
            out,
            vec![TokenKind::Utf8Started, TokenKind::Print('界')]
        );
    }

    #[test]
    fn an_interrupted_utf8_character_resolves_at_ground() {
        // `input_stop_utf8` is reached from the print and C0 handlers and from
        // nowhere else, so the sequence in between passes first and the
        // replacement lands after it.
        assert_eq!(
            tokens(b"\xe0\x1b[31mZ"),
            vec![
                TokenKind::Utf8Started,
                TokenKind::Csi {
                    private: None,
                    params: vec![Param {
                        value: Some(31),
                        subs: Vec::new()
                    }],
                    intermediates: Vec::new(),
                    final_byte: b'm',
                },
                TokenKind::Replacement,
                TokenKind::Print('Z'),
            ]
        );
        // A continuation byte after the sequence still continues the character
        // the sequence interrupted: nothing stopped it.
        assert_eq!(
            tokens("\u{754c}".as_bytes()[..1].iter().copied()
                .chain(b"\x1b[m".iter().copied())
                .chain("\u{754c}".as_bytes()[1..].iter().copied())
                .collect::<Vec<u8>>()
                .as_slice())
                .last(),
            Some(&TokenKind::Print('\u{754c}'))
        );
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
    fn a_c0_inside_a_sequence_runs_without_ending_it() {
        // `input_c0_dispatch` is reached from a table entry with a NULL target
        // state, so neither the state nor `param_buf` moves: the bell rings and
        // the CSI carries on collecting across it.
        assert_eq!(
            tokens(b"\x1b[2\x07;3H"),
            vec![
                TokenKind::Control(0x07),
                TokenKind::Csi {
                    private: None,
                    params: vec![
                        Param {
                            value: Some(2),
                            subs: Vec::new()
                        },
                        Param {
                            value: Some(3),
                            subs: Vec::new()
                        },
                    ],
                    intermediates: Vec::new(),
                    final_byte: b'H',
                },
            ]
        );
        // The parameter digits either side of the control join into one value.
        assert_eq!(
            tokens(b"\x1b[1\x082C"),
            vec![
                TokenKind::Control(0x08),
                TokenKind::Csi {
                    private: None,
                    params: vec![Param {
                        value: Some(12),
                        subs: Vec::new()
                    }],
                    intermediates: Vec::new(),
                    final_byte: b'C',
                },
            ]
        );
    }

    #[test]
    fn a_twenty_fourth_parameter_abandons_the_sequence() {
        // `param_list` holds 24 positions and `input_split` gives up once they
        // are all used, so the sequence goes rather than the excess.
        let mut payload = b"\x1b[".to_vec();
        payload.extend(std::iter::repeat_n(&b"1;"[..], 22).flatten().copied());
        payload.extend_from_slice(b"5B");
        assert_eq!(
            tokens(&payload).len(),
            1,
            "23 positions still dispatch: {payload:?}"
        );
        let mut payload = b"\x1b[".to_vec();
        payload.extend(std::iter::repeat_n(&b"1;"[..], 23).flatten().copied());
        payload.extend_from_slice(b"5B");
        assert!(tokens(&payload).is_empty(), "24 positions drop the sequence");
    }

    #[test]
    fn a_parameter_too_wide_for_an_int_abandons_the_sequence() {
        // `strtonum(out, 0, INT_MAX, &errstr)` fails, and `input_split` returns
        // -1 the moment one position does not convert.
        assert!(tokens(b"\x1b[99999999999999999999B").is_empty());
        assert!(tokens(b"\x1b[2147483648B").is_empty());
        assert_eq!(tokens(b"\x1b[2147483647B").len(), 1, "INT_MAX still fits");
        // A position with sub-parameters is never read as a number, so its
        // width is nobody's business here.
        assert_eq!(tokens(b"\x1b[99999999999999999999:1B").len(), 1);
    }

    #[test]
    fn csi_ignore_swallows_the_rest_of_the_sequence() {
        assert_eq!(tokens(b"\x1b[1;2<Hx"), vec![TokenKind::Print('x')]);
    }
}

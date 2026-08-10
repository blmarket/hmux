//! Applying tokens to the screen: tmux's `input.c` dispatch tables.
//!
//! The framing is already done by [`crate::parser`]; what is left is the
//! meaning of each sequence, which is exactly what `input.c`'s `input_c0_*`,
//! `input_esc_*` and `input_csi_*` handlers decide. They are ported here one
//! for one, against the same `screen-write.c` operations.
//!
//! Sequences that are answered rather than applied — device attributes, DSR,
//! DECRQM, OSC colours, clipboard, passthrough — are absent by design:
//! [`crate::observer`] already handled them before the token got here. What
//! remains is everything that lands on screen state, the title included.

use super::cell::{attr, colour, Cell, CellData};
use super::grid::line_flag;
use super::hyperlinks;
use super::screen::{erase_background, Screen};
use crate::parser::{Param, TokenKind};
use crate::screen::mode;
use crate::sixel;
use crate::vis;
use crate::width::codepoint_width;

/// The character sets `SO`/`SI` and `ESC ( 0` select between.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Charsets {
    /// Which of G0/G1 is currently mapped into GL, tmux's `cell.set`.
    selected: u8,
    /// Whether G0 and G1 are the line-drawing set, tmux's `g0set`/`g1set`.
    g0_acs: bool,
    g1_acs: bool,
}

impl Charsets {
    fn acs_selected(self) -> bool {
        if self.selected == 0 {
            self.g0_acs
        } else {
            self.g1_acs
        }
    }
}

/// The screen plus the parser-side state `input.c` keeps beside it.
pub struct Engine {
    pub screen: Screen,
    charsets: Charsets,
    /// The charset half of tmux's `old_cell`. DECSC saves the whole
    /// `input_cell` — the pen *and* which sets G0/G1 hold — so DECRC puts the
    /// pane back into line drawing if that is where it was. There is no
    /// "nothing saved yet" state: `input_reset` zeroes `old_cell` along with
    /// `cell`, so an unpaired DECRC restores the defaults.
    saved_charsets: Charsets,
    /// tmux's `ictx->last`: the character REP repeats.
    last: Option<CellData>,
    /// tmux's `INPUT_LAST`: whether `last` is still the most recent thing
    /// written, which is what makes REP legal.
    last_valid: bool,
}

impl Engine {
    pub fn new(sx: usize, sy: usize, hlimit: usize) -> Engine {
        Engine {
            screen: Screen::new(sx, sy, hlimit),
            charsets: Charsets::default(),
            saved_charsets: Charsets::default(),
            last: None,
            last_valid: false,
        }
    }

    /// Apply one token.
    pub fn apply(&mut self, kind: &TokenKind) {
        // "input_parse will end the collection when anything that isn't a plain
        // character is encountered" — the comment in `screen_write_collect_add`,
        // and the reason a run's extent stops where the next sequence begins.
        if !matches!(kind, TokenKind::Print(_)) {
            self.screen.end_run();
        }
        match kind {
            TokenKind::Print(character) => self.print(*character, true),
            // `input_top_bit_set` clears `INPUT_LAST` for every top-bit-set
            // byte and puts it back only for a character that completed, so
            // both an unfinished sequence and the replacement a broken one
            // resolves to leave REP with nothing to repeat.
            TokenKind::Utf8Started => self.last_valid = false,
            TokenKind::Replacement => {
                self.print('\u{fffd}', false);
                self.last_valid = false;
            }
            TokenKind::Control(byte) => self.control(*byte),
            TokenKind::Esc {
                intermediates,
                final_byte,
            } => self.esc(intermediates, *final_byte),
            TokenKind::Csi {
                private,
                params,
                intermediates,
                final_byte,
            } => self.csi(*private, params, intermediates, *final_byte),
            // Entering a string state — `input_enter_osc` and its DCS, APC and
            // rename counterparts — clears `INPUT_LAST` before a byte of the
            // string is read.
            TokenKind::Osc { data, .. } => {
                self.last_valid = false;
                self.osc(data);
            }
            TokenKind::Dcs {
                params,
                intermediates,
                final_byte,
                data,
            } => {
                self.last_valid = false;
                self.dcs(params.as_deref(), intermediates, *final_byte, data);
            }
            // The rename and APC forms never reach the screen: the observer
            // consumed them. Entering their string states still costs REP its
            // character.
            TokenKind::Apc { .. } | TokenKind::Rename { .. } => self.last_valid = false,
        }
    }

    /// tmux's `input_dcs_dispatch`, minus everything the observer already
    /// answered: what is left that touches the grid is a sixel image.
    ///
    /// tmux tests `buf[0] == 'q'` on a payload that still has the sequence's
    /// final byte at its head; hmux's parser has already split that off, so the
    /// same test is on the final byte itself.
    fn dcs(&mut self, params: Option<&[Param]>, intermediates: &[u8], final_byte: u8, data: &[u8]) {
        if final_byte != b'q' || !intermediates.is_empty() {
            return;
        }
        // `input_split` refusing the parameters stops the sixel branch, and
        // only it: the passthrough and DECRQSS branches never look at them.
        let Some(params) = params else {
            return;
        };
        // The background-select mode, `input_get(ictx, 1, 0, 0)`.
        let p2 = params.get(1).and_then(|param| param.get(0)).unwrap_or(0);
        let options = self.screen.options;
        let Some(image) = sixel::parse(data, p2, options.xpixel, options.ypixel) else {
            return;
        };
        // The pen's background, not the erase background: tmux passes
        // `ictx->cell.cell.bg` straight through.
        let bg = self.screen.cell.bg;
        self.screen.sixel_image(image, bg);
    }

    /// The background an erase uses, tmux's `bg` in `input_csi_dispatch`.
    fn bg(&self) -> i32 {
        erase_background(&self.screen.cell)
    }

    // ---- printable characters --------------------------------------------

    /// tmux's `input_print` and `input_top_bit_set`, which differ only in where
    /// the character came from.
    fn print(&mut self, character: char, records_last: bool) {
        let width = codepoint_width(character as u32);
        let mut cell = self.screen.cell.clone();
        if self.charsets.acs_selected() && character.is_ascii() {
            cell.attr |= attr::CHARSET;
        } else {
            cell.attr &= !attr::CHARSET;
        }
        cell.data = CellData::from_char(character, width);
        self.screen.put_cell(&cell);
        if records_last {
            self.last = Some(cell.data);
            self.last_valid = true;
        }
    }

    /// tmux's `input_c0_dispatch`.
    fn control(&mut self, byte: u8) {
        match byte {
            // NUL and BEL change nothing on the screen; the bell is the
            // server's business and the observer already reported it.
            0x00 | 0x07 => {}
            0x08 => self.screen.backspace(),
            0x09 => self.screen.tab(),
            0x0a..=0x0c => {
                let bg = self.screen.cell.bg;
                self.screen.linefeed(false, bg);
                if self.screen.mode & mode::CRLF != 0 {
                    self.screen.carriage_return();
                }
            }
            0x0d => self.screen.carriage_return(),
            // SO and SI swap which character set is mapped in.
            0x0e => self.charsets.selected = 1,
            0x0f => self.charsets.selected = 0,
            _ => {}
        }
        self.last_valid = false;
    }

    // ---- saved state ------------------------------------------------------

    /// tmux's `input_save_state`, behind DECSC and SCP.
    fn save_state(&mut self) {
        self.saved_charsets = self.charsets;
        self.screen.save_cursor();
    }

    /// tmux's `input_restore_state`, behind DECRC and RCP.
    fn restore_state(&mut self) {
        self.charsets = self.saved_charsets;
        self.screen.restore_cursor();
    }

    // ---- escape sequences -------------------------------------------------

    /// tmux's `input_esc_dispatch`.
    fn esc(&mut self, intermediates: &[u8], final_byte: u8) {
        match (intermediates, final_byte) {
            // RIS.
            ([], b'c') => {
                self.charsets = Charsets::default();
                self.saved_charsets = Charsets::default();
                self.screen.reset();
            }
            // IND.
            ([], b'D') => {
                let bg = self.screen.cell.bg;
                self.screen.linefeed(false, bg);
            }
            // NEL.
            ([], b'E') => {
                let bg = self.screen.cell.bg;
                self.screen.carriage_return();
                self.screen.linefeed(false, bg);
            }
            // HTS: the observer reports the stop, and the screen records it.
            ([], b'H') if self.screen.cx < self.screen.sx() => {
                let cx = self.screen.cx;
                self.screen.tabs.insert(cx);
            }
            // RI.
            ([], b'M') => {
                let bg = self.screen.cell.bg;
                self.screen.reverse_index(bg);
            }
            // DECKPAM / DECKPNM.
            ([], b'=') => self.screen.mode_set(mode::KKEYPAD),
            ([], b'>') => self.screen.mode_clear(mode::KKEYPAD),
            // DECSC / DECRC.
            ([], b'7') => self.save_state(),
            ([], b'8') => self.restore_state(),
            // DECALN.
            ([b'#'], b'8') => self.screen.alignment_test(),
            // SCS: select the line-drawing set into G0 or G1, or ASCII back.
            ([b'('], b'0') => self.charsets.g0_acs = true,
            ([b'('], b'B') => self.charsets.g0_acs = false,
            ([b')'], b'0') => self.charsets.g1_acs = true,
            ([b')'], b'B') => self.charsets.g1_acs = false,
            _ => {}
        }
        // `input_esc_dispatch` returns before this when the sequence is not in
        // its table, so an escape tmux does not recognise leaves REP's
        // character where it was.
        if esc_is_known(intermediates, final_byte) {
            self.last_valid = false;
        }
    }

    // ---- control sequences ------------------------------------------------

    #[allow(clippy::too_many_lines)]
    fn csi(&mut self, private: Option<u8>, params: &[Param], intermediates: &[u8], final_byte: u8) {
        let bg = self.bg();
        // tmux's `input_get`. `None` is its -1, which a position carrying
        // sub-parameters answers with; every handler below skips its operation
        // rather than acting on the part before the colon.
        let get = |index: usize, default: u32| -> Option<usize> {
            params
                .get(index)
                .map_or(Some(default), |param| param.get(default))
                .map(|value| value as usize)
        };
        // The same reading with a minimum of one: a zero parameter is read as
        // one for every operation that counts something.
        let count = |index: usize| -> Option<usize> { get(index, 1).map(|value| value.max(1)) };

        match (private, intermediates, final_byte) {
            // ICH.
            (None, [], b'@') => {
                if let Some(n) = count(0) {
                    self.screen.insert_character(n, bg);
                }
            }
            // CUU / CUD / CUF / CUB.
            (None, [], b'A') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_up(n);
                }
            }
            (None, [], b'B') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_down(n);
                }
            }
            (None, [], b'C') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_right(n);
                }
            }
            (None, [], b'D') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_left(n);
                }
            }
            // CNL / CPL.
            (None, [], b'E') => {
                if let Some(n) = count(0) {
                    self.screen.carriage_return();
                    self.screen.cursor_down(n);
                }
            }
            (None, [], b'F') => {
                if let Some(n) = count(0) {
                    self.screen.carriage_return();
                    self.screen.cursor_up(n);
                }
            }
            // HPA (both spellings).
            (None, [], b'G' | b'`') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_move(Some(n - 1), None, true);
                }
            }
            // CUP / HVP.
            (None, [], b'H' | b'f') => {
                if let (Some(row), Some(column)) = (count(0), count(1)) {
                    self.screen.cursor_move(Some(column - 1), Some(row - 1), true);
                }
            }
            // ED.
            (None, [], b'J') => match get(0, 0) {
                Some(0) => self.screen.clear_end_of_screen(bg),
                Some(1) => self.screen.clear_start_of_screen(bg),
                Some(2) => self.screen.clear_screen(bg),
                // A Linux console extension: drop the history.
                Some(3) if get(1, 0) == Some(0) => self.screen.clear_history(),
                _ => {}
            },
            // EL.
            (None, [], b'K') => match get(0, 0) {
                Some(0) => self.screen.clear_end_of_line(bg),
                Some(1) => self.screen.clear_start_of_line(bg),
                Some(2) => self.screen.clear_line(bg),
                _ => {}
            },
            // IL / DL.
            (None, [], b'L') => {
                if let Some(n) = count(0) {
                    self.screen.insert_line(n, bg);
                }
            }
            (None, [], b'M') => {
                if let Some(n) = count(0) {
                    self.screen.delete_line(n, bg);
                }
            }
            // DCH.
            (None, [], b'P') => {
                if let Some(n) = count(0) {
                    self.screen.delete_character(n, bg);
                }
            }
            // SU / SD.
            (None, [], b'S') => {
                if let Some(n) = count(0) {
                    self.screen.scroll_up(n, bg);
                }
            }
            (None, [], b'T') => {
                if let Some(n) = count(0) {
                    self.screen.scroll_down(n, bg);
                }
            }
            // ECH.
            (None, [], b'X') => {
                if let Some(n) = count(0) {
                    self.screen.clear_character(n, bg);
                }
            }
            // CBT: back to the previous tab stop, n times.
            (None, [], b'Z') => {
                if let Some(n) = count(0) {
                    self.cursor_back_tab(n);
                }
            }
            // REP: repeat the last printable character.
            (None, [], b'b') => {
                if let Some(n) = count(0) {
                    self.repeat(n);
                }
            }
            // VPA.
            (None, [], b'd') => {
                if let Some(n) = count(0) {
                    self.screen.cursor_move(None, Some(n - 1), true);
                }
            }
            // TBC.
            (None, [], b'g') => match get(0, 0) {
                Some(0) => {
                    let cx = self.screen.cx;
                    self.screen.tabs.remove(&cx);
                }
                Some(3) => self.screen.tabs.clear(),
                _ => {}
            },
            // SM / RM.
            (None, [], b'h') => self.set_modes(params, true),
            (None, [], b'l') => self.set_modes(params, false),
            // DECSET / DECRST.
            (Some(b'?'), [], b'h') => self.set_private_modes(params, true),
            (Some(b'?'), [], b'l') => self.set_private_modes(params, false),
            // SGR.
            (None, [], b'm') => self.sgr(params),
            // DECSTBM.
            (None, [], b'r') => {
                let sy = self.screen.sy();
                if let (Some(upper), Some(lower)) = (count(0), get(1, sy as u32)) {
                    // tmux homes the cursor from inside `screen_write_scrollregion`,
                    // so a refused region leaves it where it was. The home is
                    // `screen_write_set_cursor`, which is absolute: the region
                    // has only just been named and origin mode does not bend
                    // this move into it. DECOM's own home does that, when it
                    // arrives.
                    if self.screen.set_scroll_region(upper - 1, lower.max(1) - 1) {
                        self.screen.set_cursor(Some(0), Some(0));
                    }
                }
            }
            // SCP / RCP, the CSI spellings of DECSC and DECRC.
            (None, [], b's') => self.save_state(),
            (None, [], b'u') => self.restore_state(),
            // `CSI > 4 ; n m`, tmux's MODKEYS: which `modifyOtherKeys` level
            // the pane wants. Only the two bits are recorded here; whether the
            // pane actually gets them is `extended-keys`'s decision, made where
            // the key is encoded.
            (Some(b'>'), [], b'm') if params.first().is_none_or(|param| param.get(4) == Some(4)) => {
                self.screen.mode_clear(mode::ALL_KEYS_EXTENDED);
                match get(1, 0) {
                    Some(1) => self.screen.mode_set(mode::KEYS_EXTENDED),
                    Some(2) => self.screen.mode_set(mode::KEYS_EXTENDED_2),
                    _ => {}
                }
            }
            // `CSI > 4 n`, tmux's MODOFF: withdraw the request.
            (Some(b'>'), [], b'n') if get(0, 0) == Some(4) => {
                self.screen.mode_clear(mode::ALL_KEYS_EXTENDED);
            }
            // XTWINOPS, walked with tmux's argument grammar. The geometry
            // reports are the observer's to answer; what the screen keeps is
            // the title stack `CSI 22 t` pushes onto and `CSI 23 t` pops from.
            // Only the window title and the "both" spelling (0 and 2) reach
            // the stack; the icon-name-only form is understood and does
            // nothing. A missing argument abandons the rest of the sequence,
            // as tmux's `case -1` does.
            (None, [], b't') => {
                let read = |index: usize| -> Option<u32> {
                    params
                        .get(index)
                        .filter(|param| !param.is_string())
                        .and_then(|param| param.value)
                };
                let mut index = 0;
                while let Some(operation) = read(index) {
                    match operation {
                        // Two arguments: move, resize in pixels, resize in
                        // cells.
                        3 | 4 | 8 => {
                            index += 2;
                            if read(index - 1).is_none() || read(index).is_none() {
                                break;
                            }
                        }
                        // One argument: raise/lower, refresh.
                        9 | 10 => {
                            index += 1;
                            if read(index).is_none() {
                                break;
                            }
                        }
                        22 | 23 => {
                            index += 1;
                            match read(index) {
                                None => break,
                                Some(0 | 2) if operation == 22 => self.screen.push_title(),
                                Some(0 | 2) => self.screen.pop_title(),
                                Some(_) => {}
                            }
                        }
                        _ => {}
                    }
                    index += 1;
                }
            }
            // DECSCUSR: the style, plus the blink every style but the default
            // decides. Style 0 goes back to the default without touching the
            // blink, as tmux's `screen_set_cursor_style` case 0 does.
            (None, [b' '], b'q') => {
                if let Some(style) = get(0, 0) {
                    if style <= 6 {
                        self.screen.cursor_style = style as u8;
                    }
                    if (1..=6).contains(&style) {
                        self.toggle(mode::CURSOR_BLINKING, style % 2 == 1);
                    }
                }
            }
            _ => {}
        }
        // Every sequence in `input_csi_table` clears `INPUT_LAST` on the way
        // out — REP included, so a second REP straight after the first has
        // nothing left to repeat. One that is not in the table returns before
        // that and leaves the character alone.
        if csi_is_known(private, intermediates, final_byte) {
            self.last_valid = false;
        }
    }

    /// CBT.
    fn cursor_back_tab(&mut self, n: usize) {
        let mut cx = self.screen.cx.min(self.screen.sx() - 1);
        for _ in 0..n {
            if cx == 0 {
                break;
            }
            loop {
                cx -= 1;
                if cx == 0 || self.screen.tabs.contains(&cx) {
                    break;
                }
            }
        }
        self.screen.cx = cx;
    }

    /// REP: write the last printable character `n` more times, clamped to the
    /// rest of the row. Anything other than a printable character in between
    /// makes it a no-op, as tmux's `INPUT_LAST` flag does.
    fn repeat(&mut self, n: usize) {
        if !self.last_valid {
            return;
        }
        let Some(data) = self.last.clone() else {
            return;
        };
        let n = n.min(self.screen.sx() - self.screen.cx);
        // REP reapplies the active character set, as printing does — the
        // repeated cell is drawn from the line-drawing set when SO is in
        // effect, not from ASCII. Unlike `input_print`, `INPUT_CSI_REP` leaves
        // the attribute on the pending cell rather than clearing it again, so
        // this writes through `screen.cell` instead of a copy of it.
        if self.charsets.acs_selected() {
            self.screen.cell.attr |= attr::CHARSET;
        } else {
            self.screen.cell.attr &= !attr::CHARSET;
        }
        self.screen.cell.data = data;
        let cell = self.screen.cell.clone();
        for _ in 0..n {
            self.screen.put_cell(&cell);
        }
    }

    /// SM / RM, the non-private modes. tmux implements only IRM.
    fn set_modes(&mut self, params: &[Param], set: bool) {
        for param in params {
            if param.get(0) == Some(4) {
                if set {
                    self.screen.mode_set(mode::INSERT);
                } else {
                    self.screen.mode_clear(mode::INSERT);
                }
            }
        }
    }

    /// DECSET / DECRST.
    ///
    /// Every mode the pane can set lands here, including the ones only the
    /// server reads. The screen is where tmux keeps them and where hmux keeps
    /// them: one sequence writes a mode and one word holds it.
    fn set_private_modes(&mut self, params: &[Param], set: bool) {
        for param in params {
            // tmux reads each position with a default of -1, so an empty one is
            // skipped rather than defaulted — and so is one carrying
            // sub-parameters, which `input_get` answers -1 for.
            if param.is_string() {
                continue;
            }
            let Some(mode) = param.value else { continue };
            match mode {
                // DECCKM.
                1 => self.toggle(mode::KCURSOR, set),
                // DECCOLM. A pane's width is the layout's to decide, so the
                // column count has nowhere to land — but the clear the mode
                // implies still happens, either way round, with the cursor
                // brought home first.
                3 => {
                    self.screen.cursor_move(Some(0), Some(0), true);
                    let bg = self.bg();
                    self.screen.clear_screen(bg);
                }
                // DECOM. Setting or clearing it homes the cursor.
                6 => {
                    self.toggle(mode::ORIGIN, set);
                    self.screen.cursor_move(Some(0), Some(0), true);
                }
                // DECAWM.
                7 => self.toggle(mode::WRAP, set),
                // The cursor blink. Either spelling also records that the pane
                // asked at all, which is what tells a later query apart from
                // the `cursor-style` option's answer.
                12 => {
                    self.toggle(mode::CURSOR_BLINKING, set);
                    self.screen.mode_set(mode::CURSOR_BLINKING_SET);
                }
                // DECTCEM.
                25 => self.toggle(mode::CURSOR, set),
                // The mouse reporting modes. tmux keeps 1000/1002/1003
                // mutually exclusive — each one clears the others — and tracks
                // the two encodings independently.
                1000 | 1002 | 1003 => {
                    self.screen.mode_clear(mode::ALL_MOUSE);
                    if set {
                        self.screen.mode_set(match mode {
                            1000 => mode::MOUSE_STANDARD,
                            1002 => mode::MOUSE_BUTTON,
                            _ => mode::MOUSE_ALL,
                        });
                    }
                }
                // Highlight tracking, which tmux does not implement: turning it
                // off still turns the modes it groups with off, and turning it
                // on does nothing.
                1001 if !set => self.screen.mode_clear(mode::ALL_MOUSE),
                1004 => self.toggle(mode::FOCUSON, set),
                1005 => self.toggle(mode::MOUSE_UTF8, set),
                1006 => self.toggle(mode::MOUSE_SGR, set),
                2004 => self.toggle(mode::BRACKETPASTE, set),
                2026 => self.toggle(mode::SYNC, set),
                2031 => self.toggle(mode::THEME_UPDATES, set),
                // The alternate screen. 1049 also saves and restores the
                // cursor; 47 and 1047 do not.
                47 | 1047 => {
                    if set {
                        self.screen.alternate_on(false);
                    } else {
                        self.screen.alternate_off(false);
                    }
                }
                1049 => {
                    if set {
                        self.screen.alternate_on(true);
                    } else {
                        self.screen.alternate_off(true);
                    }
                }
                _ => {}
            }
        }
    }

    fn toggle(&mut self, bits: u32, set: bool) {
        if set {
            self.screen.mode_set(bits);
        } else {
            self.screen.mode_clear(bits);
        }
    }

    // ---- SGR --------------------------------------------------------------

    /// tmux's `input_csi_dispatch_sgr`.
    fn sgr(&mut self, params: &[Param]) {
        // `CSI m` with no parameters at all copies the default cell wholesale,
        // which drops the open hyperlink with everything else. `CSI 0 m` goes
        // through the loop below, where tmux lifts the link out and puts it
        // back — so the two spellings of "reset" are not the same sequence.
        if params.is_empty() {
            self.screen.cell = Cell::default();
            return;
        }
        let mut index = 0;
        while index < params.len() {
            let param = &params[index];
            // A parameter with sub-parameters is the colon form, which carries
            // its own arguments rather than taking the ones after it.
            if !param.subs.is_empty() {
                self.sgr_colon(param);
                index += 1;
                continue;
            }
            let n = param.value.unwrap_or(0);
            if matches!(n, 38 | 48 | 58) {
                index += 1;
                let at = |offset: usize| {
                    params
                        .get(index + offset)
                        .filter(|param| !param.is_string())
                        .and_then(|param| param.value)
                };
                // `input_get(ictx, i, 0, -1)`: a position left empty and one
                // carrying sub-parameters both read as -1 and select neither
                // arm, leaving the introducer to be skipped on its own.
                match at(0) {
                    // Direct colour: three more parameters — but only if the
                    // colour is accepted. tmux advances past them from inside
                    // `input_csi_dispatch_sgr_rgb`, so a rejected colour leaves
                    // them to be read as ordinary SGR parameters.
                    Some(2) => {
                        let applied = self.sgr_rgb(n, at(1), at(2), at(3));
                        if applied {
                            index += 3;
                        }
                    }
                    // Palette colour: one more, consumed either way.
                    Some(5) => {
                        self.sgr_indexed(n, at(1));
                        index += 1;
                    }
                    _ => {}
                }
                index += 1;
                continue;
            }
            self.sgr_one(n);
            index += 1;
        }
    }

    /// The colon form, `38:2::r:g:b` and `4:3`.
    fn sgr_colon(&mut self, param: &Param) {
        // `input_csi_dispatch_sgr_colon` splits into a fixed array of eight and
        // abandons the whole parameter the moment it has counted the eighth
        // position — before it looks at what any of them said. A colon list
        // that long therefore does nothing at all, however well-formed its
        // first positions were.
        if 1 + param.subs.len() >= 8 {
            return;
        }
        let read = |index: usize| -> Option<u32> { param.subs.get(index).copied().flatten() };
        match param.value {
            // Underline style.
            Some(4) => {
                if param.subs.len() != 1 {
                    return;
                }
                let cell = &mut self.screen.cell;
                cell.attr &= !attr::ALL_UNDERSCORE;
                cell.attr |= match read(0) {
                    Some(1) => attr::UNDERSCORE,
                    Some(2) => attr::UNDERSCORE_2,
                    Some(3) => attr::UNDERSCORE_3,
                    Some(4) => attr::UNDERSCORE_4,
                    Some(5) => attr::UNDERSCORE_5,
                    _ => 0,
                };
            }
            Some(n @ (38 | 48 | 58)) => match read(0) {
                // `38:2::r:g:b` has an empty colour-space id; `38:2:r:g:b`
                // does not. Both are in use, and tmux tells them apart by the
                // count of positions rather than by looking for the gap.
                Some(2) => {
                    let count = 1 + param.subs.len();
                    let first = if count == 5 { 2 } else { 3 };
                    if count >= first + 3 {
                        self.sgr_rgb(n, read(first - 1), read(first), read(first + 1));
                    }
                }
                // tmux's `n < 3`: the palette index is the third position, so
                // a shorter list names no colour at all.
                Some(5) if param.subs.len() >= 2 => self.sgr_indexed(n, read(1)),
                _ => {}
            },
            _ => {}
        }
    }

    /// tmux's `input_csi_dispatch_sgr_256_do`. An index past 255 is not a
    /// palette entry, and neither is a position left empty; tmux resets the
    /// colour to the default rather than folding the value into a byte.
    ///
    /// The reset arm covers only 38 and 48 — tmux has no 58 case on that side
    /// of the test, so an out-of-range underline colour leaves the pen alone.
    fn sgr_indexed(&mut self, which: u32, value: Option<u32>) {
        match value.filter(|value| *value <= 255) {
            Some(value) => self.set_sgr_colour(which, colour::indexed(value as u8)),
            None if which != 58 => self.set_sgr_colour(which, colour::DEFAULT),
            None => {}
        }
    }

    /// tmux's `input_csi_dispatch_sgr_rgb_do`. The colour is applied only when
    /// all three components are present and fit a byte; the answer is also what
    /// tells the semicolon form whether those positions were arguments at all.
    fn sgr_rgb(
        &mut self,
        which: u32,
        red: Option<u32>,
        green: Option<u32>,
        blue: Option<u32>,
    ) -> bool {
        let byte = |value: Option<u32>| value.filter(|value| *value <= 255);
        let (Some(red), Some(green), Some(blue)) = (byte(red), byte(green), byte(blue)) else {
            return false;
        };
        self.set_sgr_colour(which, colour::rgb(red as u8, green as u8, blue as u8));
        true
    }

    fn set_sgr_colour(&mut self, which: u32, value: i32) {
        let cell = &mut self.screen.cell;
        match which {
            38 => cell.fg = value,
            48 => cell.bg = value,
            _ => cell.us = value,
        }
    }

    /// One plain SGR parameter.
    fn sgr_one(&mut self, n: u32) {
        let cell = &mut self.screen.cell;
        match n {
            0 => {
                let link = cell.link;
                *cell = Cell {
                    link,
                    ..Cell::default()
                };
            }
            1 => cell.attr |= attr::BRIGHT,
            2 => cell.attr |= attr::DIM,
            3 => cell.attr |= attr::ITALICS,
            4 => {
                cell.attr &= !attr::ALL_UNDERSCORE;
                cell.attr |= attr::UNDERSCORE;
            }
            5 | 6 => cell.attr |= attr::BLINK,
            7 => cell.attr |= attr::REVERSE,
            8 => cell.attr |= attr::HIDDEN,
            9 => cell.attr |= attr::STRIKETHROUGH,
            21 => {
                cell.attr &= !attr::ALL_UNDERSCORE;
                cell.attr |= attr::UNDERSCORE_2;
            }
            22 => cell.attr &= !(attr::BRIGHT | attr::DIM),
            23 => cell.attr &= !attr::ITALICS,
            24 => cell.attr &= !attr::ALL_UNDERSCORE,
            25 => cell.attr &= !attr::BLINK,
            27 => cell.attr &= !attr::REVERSE,
            28 => cell.attr &= !attr::HIDDEN,
            29 => cell.attr &= !attr::STRIKETHROUGH,
            30..=37 => cell.fg = (n - 30) as i32,
            39 => cell.fg = colour::DEFAULT,
            40..=47 => cell.bg = (n - 40) as i32,
            49 => cell.bg = colour::DEFAULT,
            53 => cell.attr |= attr::OVERLINE,
            55 => cell.attr &= !attr::OVERLINE,
            59 => cell.us = colour::DEFAULT,
            90..=97 => cell.fg = n as i32,
            100..=107 => cell.bg = (n - 10) as i32,
            _ => {}
        }
    }

    // ---- OSC --------------------------------------------------------------

    /// The OSC sequences that change the *grid* are the shell-integration ones
    /// and the hyperlink one. Everything else the observer already turned into
    /// server state.
    fn osc(&mut self, data: &[u8]) {
        // tmux's `input_exit_osc` number parse: a body that does not open with
        // digits names no command, and a run of digits too long for a `u_int`
        // wraps rather than failing. The bytes stay bytes past the number: tmux
        // never asks whether a URI or a title is text before looking at it.
        let digits = data.iter().take_while(|byte| byte.is_ascii_digit()).count();
        if digits == 0 {
            return;
        }
        let number = data[..digits].iter().fold(0u32, |number, digit| {
            number
                .wrapping_mul(10)
                .wrapping_add(u32::from(digit - b'0'))
        });
        let body = match data[digits..].first() {
            None => &[][..],
            Some(b';') => &data[digits + 1..],
            // Anything else is not a well-formed OSC.
            Some(_) => return,
        };
        match number {
            // The title. The `allow-set-title` gate has already dropped the
            // sequence upstream; what is applied here is tmux's
            // `screen_set_title`, whose clean refuses a name it cannot vis and
            // leaves the one already set.
            0 | 2 => {
                if let Some(title) = vis::clean_name(body) {
                    self.screen.title = Some(title);
                }
                return;
            }
            8 => {
                self.osc_8(body);
                return;
            }
            133 => {}
            _ => return,
        }
        let row = self.screen.grid.hsize + self.screen.cy;
        match body.first().copied().map(char::from) {
            // A prompt starts here.
            Some('A') => self
                .screen
                .grid
                .set_line_flags(row, line_flag::START_PROMPT, true),
            // Command output starts here.
            Some('C') => self
                .screen
                .grid
                .set_line_flags(row, line_flag::START_OUTPUT, true),
            _ => {}
        }
    }

    /// tmux's `input_osc_8`: open or close a hyperlink on the pen, so the cells
    /// written after it belong to that link.
    fn osc_8(&mut self, body: &[u8]) {
        // A malformed sequence leaves whatever link is open alone, as tmux's
        // `bad` path does.
        let Some((id, uri)) = hyperlinks::parse(body) else {
            return;
        };
        self.screen.cell.link = match uri {
            Some(uri) => self.screen.hyperlinks.put(uri, id),
            // An empty URI closes the open link.
            None => 0,
        };
    }
}

/// Whether this escape sequence is in tmux's `input_esc_table`.
///
/// `input_esc_dispatch` looks the sequence up before it does anything, and a
/// miss returns straight away — which is why what the table holds is observable
/// beyond the operations themselves: only a hit clears `INPUT_LAST`.
fn esc_is_known(intermediates: &[u8], final_byte: u8) -> bool {
    matches!(
        (intermediates, final_byte),
        ([], b'7' | b'8' | b'=' | b'>' | b'D' | b'E' | b'H' | b'M' | b'\\' | b'c')
            | ([b'#'], b'8')
            | ([b'('] | [b')'], b'0' | b'B')
    )
}

/// Whether this control sequence is in tmux's `input_csi_table`.
///
/// tmux keeps the private marker in the same intermediate buffer the table is
/// keyed on, so `?$p` is one entry and `$p` another; here the two arrive apart
/// and are paired up again.
fn csi_is_known(private: Option<u8>, intermediates: &[u8], final_byte: u8) -> bool {
    matches!(
        (private, intermediates, final_byte),
        (
            None,
            [],
            b'@' | b'A'
                | b'B'
                | b'C'
                | b'D'
                | b'E'
                | b'F'
                | b'G'
                | b'H'
                | b'J'
                | b'K'
                | b'L'
                | b'M'
                | b'P'
                | b'S'
                | b'T'
                | b'X'
                | b'Z'
                | b'`'
                | b'b'
                | b'c'
                | b'd'
                | b'f'
                | b'g'
                | b'h'
                | b'l'
                | b'm'
                | b'n'
                | b'r'
                | b's'
                | b't'
                | b'u'
        ) | (Some(b'?'), [], b'S' | b'h' | b'l' | b'n')
            | (Some(b'>'), [], b'c' | b'm' | b'n' | b'q')
            | (None, [b'$'], b'p')
            | (Some(b'?'), [b'$'], b'p')
            | (None, [b' '], b'q')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    fn screen(sx: usize, sy: usize) -> Engine {
        Engine::new(sx, sy, 100)
    }

    fn feed(engine: &mut Engine, input: &[u8]) {
        for token in tokenize(input) {
            engine.apply(&token.kind);
        }
    }

    fn text(engine: &Engine, py: usize) -> String {
        let grid = &engine.screen.grid;
        let row = grid.hsize + py;
        (0..grid.line_length(row))
            .map(|px| grid.get(px, row).data.text().to_string())
            .collect()
    }

    /// `INPUT_CSI_REP` reapplies the active character set before it repeats,
    /// so a repeat under SO draws from the line-drawing set — and it leaves the
    /// attribute on the pending cell, where `input_print` clears it again.
    #[test]
    fn rep_repeats_under_the_active_character_set() {
        let mut engine = screen(10, 1);
        feed(&mut engine, b"\x1b(0q\x1b[2b");
        for px in 0..3 {
            assert_eq!(
                engine.screen.grid.get(px, 0).attr & attr::CHARSET,
                attr::CHARSET,
                "cell {px} is drawn from the line-drawing set"
            );
        }

        let mut ascii = screen(10, 1);
        feed(&mut ascii, b"q\x1b[2b");
        for px in 0..3 {
            assert_eq!(ascii.screen.grid.get(px, 0).attr & attr::CHARSET, 0);
        }
    }

    /// The cluster in a cell and the cursor column, which together are what
    /// the combining rules are observable through.
    fn cell(engine: &Engine, px: usize) -> String {
        engine
            .screen
            .grid
            .get(px, engine.screen.grid.hsize)
            .data
            .text()
            .to_string()
    }

    // The expectations in the combining tests below were taken from tmux 3.7b
    // itself: each payload was printed into a clean pane and the cursor column
    // and `capture-pane` bytes read back. They are the oracle's answers, not a
    // reading of `utf8-combined.c`.

    /// The snapshot's view of one viewport cell, which is what `capture-pane`
    /// and copy mode read.
    fn snapshot_cell(engine: &Engine, px: usize) -> crate::screen::GridCell {
        let grid = super::super::dump::snapshot(&engine.screen, engine.screen.grid.hsize, 1);
        grid.rows[0].cells[px].clone()
    }

    #[test]
    fn an_osc_8_link_is_carried_by_the_cells_written_under_it() {
        let mut engine = screen(24, 3);
        feed(
            &mut engine,
            b"\x1b]8;id=x1;https://a.test\x1b\\LINK\x1b]8;;\x1b\\ plain",
        );
        assert_eq!(text(&engine, 0), "LINK plain");
        let slot = snapshot_cell(&engine, 0).hyperlink_slot;
        assert_ne!(slot, 0, "a linked cell carries the screen's own identity");
        for px in 0..4 {
            let cell = snapshot_cell(&engine, px);
            assert_eq!(cell.hyperlink.as_deref(), Some("https://a.test"));
            assert_eq!(cell.hyperlink_slot, slot);
        }
        // The empty URI closed the link, so what follows is not part of it.
        assert_eq!(snapshot_cell(&engine, 4).hyperlink, None);
        assert_eq!(snapshot_cell(&engine, 5).hyperlink, None);
    }

    #[test]
    fn an_anonymous_link_reports_its_uri() {
        let mut engine = screen(24, 3);
        feed(&mut engine, b"\x1b]8;;https://b.test\x1b\\X");
        let cell = snapshot_cell(&engine, 0);
        assert_eq!(cell.hyperlink.as_deref(), Some("https://b.test"));
        assert_ne!(cell.hyperlink_slot, 0);
    }

    #[test]
    fn a_row_carrying_a_link_is_flagged_for_the_capture_path() {
        let mut engine = screen(24, 3);
        feed(&mut engine, b"\x1b]8;;https://b.test\x1b\\X");
        let row = engine.screen.grid.hsize;
        assert!(engine.screen.grid.line(row).unwrap().flags & line_flag::HYPERLINK != 0);
    }

    #[test]
    fn a_malformed_osc_8_leaves_the_open_link_alone() {
        let mut engine = screen(24, 3);
        // No `;` at all: tmux logs it and changes nothing.
        feed(
            &mut engine,
            b"\x1b]8;;https://b.test\x1b\\A\x1b]8;id=x1\x1b\\B",
        );
        assert_eq!(
            snapshot_cell(&engine, 1).hyperlink.as_deref(),
            Some("https://b.test")
        );
    }

    #[test]
    fn a_hangul_filler_is_dropped_without_taking_a_cell() {
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{3164}".as_bytes());
        assert_eq!(engine.screen.cx, 0);
        assert_eq!(text(&engine, 0), "");
    }

    #[test]
    fn hangul_jamo_compose_into_one_cell_in_syllable_order() {
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{1100}\u{1161}\u{11a8}".as_bytes());
        assert_eq!(engine.screen.cx, 2, "the syllable is one wide cell");
        assert_eq!(cell(&engine, 0), "\u{1100}\u{1161}\u{11a8}");
    }

    #[test]
    fn a_jamo_with_nothing_to_compose_onto_is_discarded() {
        // tmux prints `a` and drops the vowel: it cannot begin a syllable.
        let mut engine = screen(12, 3);
        feed(&mut engine, "a\u{1161}".as_bytes());
        assert_eq!(engine.screen.cx, 1);
        assert_eq!(text(&engine, 0), "a");
    }

    #[test]
    fn a_lone_jamo_vowel_still_gets_its_own_cell_at_the_margin() {
        // At column zero there is nothing to combine with, so the vowel is
        // placed rather than dropped, and it is one column wide.
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{1161}".as_bytes());
        assert_eq!(engine.screen.cx, 1);
        assert_eq!(cell(&engine, 0), "\u{1161}");
    }

    #[test]
    fn two_regional_indicators_make_one_wide_flag() {
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{1f1ec}\u{1f1e7}".as_bytes());
        assert_eq!(engine.screen.cx, 2, "the pair is forced to two columns");
        assert_eq!(cell(&engine, 0), "\u{1f1ec}\u{1f1e7}");
        assert!(engine
            .screen
            .grid
            .get(1, engine.screen.grid.hsize)
            .is_padding());
    }

    #[test]
    fn a_third_regional_indicator_starts_the_next_flag() {
        let mut engine = screen(12, 3);
        feed(
            &mut engine,
            "\u{1f1ec}\u{1f1e7}\u{1f1ec}\u{1f1e7}".as_bytes(),
        );
        assert_eq!(engine.screen.cx, 4, "two flags, two cells");
        assert_eq!(cell(&engine, 0), "\u{1f1ec}\u{1f1e7}");
        assert_eq!(cell(&engine, 2), "\u{1f1ec}\u{1f1e7}");
    }

    #[test]
    fn a_skin_tone_modifier_joins_the_emoji_before_it() {
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{1f469}\u{1f3fd}".as_bytes());
        assert_eq!(engine.screen.cx, 2);
        assert_eq!(cell(&engine, 0), "\u{1f469}\u{1f3fd}");
    }

    #[test]
    fn a_variation_selector_widens_the_cell_it_joins() {
        // U+2764 is one column on its own; VS16 makes the pair two, which is
        // what `variation-selector-always-wide` asks for and what tmux does.
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{2764}\u{fe0f}".as_bytes());
        assert_eq!(engine.screen.cx, 2);
        assert_eq!(cell(&engine, 0), "\u{2764}\u{fe0f}");
        assert!(engine
            .screen
            .grid
            .get(1, engine.screen.grid.hsize)
            .is_padding());
    }

    #[test]
    fn a_joiner_pulls_the_next_emoji_into_the_same_cell() {
        let mut engine = screen(12, 3);
        feed(&mut engine, "\u{1f468}\u{200d}\u{1f469}".as_bytes());
        assert_eq!(engine.screen.cx, 2, "the joined pair stays one cell");
        assert_eq!(cell(&engine, 0), "\u{1f468}\u{200d}\u{1f469}");
    }

    #[test]
    fn plain_text_and_newlines_land_where_they_should() {
        let mut engine = screen(20, 3);
        feed(&mut engine, b"one\r\ntwo");
        assert_eq!(text(&engine, 0), "one");
        assert_eq!(text(&engine, 1), "two");
        assert_eq!((engine.screen.cx, engine.screen.cy), (3, 1));
    }

    #[test]
    fn cursor_addressing_is_one_based_on_the_wire() {
        let mut engine = screen(20, 5);
        feed(&mut engine, b"\x1b[3;5Hx");
        assert_eq!((engine.screen.cx, engine.screen.cy), (5, 2));
        assert_eq!(engine.screen.grid.get(4, 2).data.text(), "x");
    }

    #[test]
    fn a_missing_parameter_takes_the_sequences_own_default() {
        let mut engine = screen(20, 5);
        feed(&mut engine, b"\x1b[10;10H\x1b[H");
        assert_eq!((engine.screen.cx, engine.screen.cy), (0, 0));
        feed(&mut engine, b"\x1b[3;3H\x1b[C");
        assert_eq!(engine.screen.cx, 3, "CUF with no parameter moves one");
    }

    #[test]
    fn erase_in_line_reaches_exactly_where_the_parameter_says() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"abcdef\x1b[1;4H\x1b[K");
        assert_eq!(text(&engine, 0), "abc");

        let mut engine = screen(10, 2);
        feed(&mut engine, b"abcdef\x1b[1;4H\x1b[1K");
        assert_eq!(text(&engine, 0), "    ef");

        let mut engine = screen(10, 2);
        feed(&mut engine, b"abcdef\x1b[2K");
        assert_eq!(text(&engine, 0), "");
    }

    #[test]
    fn erase_in_display_clears_forward_backward_and_whole() {
        let mut engine = screen(6, 3);
        feed(&mut engine, b"aaa\r\nbbb\r\nccc\x1b[2;2H\x1b[J");
        assert_eq!(text(&engine, 0), "aaa");
        assert_eq!(text(&engine, 1), "b");
        assert_eq!(text(&engine, 2), "");
    }

    #[test]
    fn sgr_sets_and_resets_the_pen() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b[1;31mA\x1b[0mB");
        let a = engine.screen.grid.get(0, 0);
        assert_eq!(a.attr & attr::BRIGHT, attr::BRIGHT);
        assert_eq!(a.fg, 1);
        let b = engine.screen.grid.get(1, 0);
        assert_eq!(b.attr, 0);
        assert_eq!(b.fg, colour::DEFAULT);
    }

    #[test]
    fn sgr_carries_direct_and_palette_colours_in_both_notations() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b[38;2;1;2;3mA");
        assert_eq!(engine.screen.grid.get(0, 0).fg, colour::rgb(1, 2, 3));

        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b[38:2::4:5:6mA");
        assert_eq!(
            engine.screen.grid.get(0, 0).fg,
            colour::rgb(4, 5, 6),
            "the colon form with an empty colour-space id"
        );

        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b[48;5;200mA");
        assert_eq!(engine.screen.grid.get(0, 0).bg, colour::indexed(200));
    }

    #[test]
    fn the_underline_style_replaces_rather_than_accumulates() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b[4m\x1b[4:3mA");
        let cell = engine.screen.grid.get(0, 0);
        assert_eq!(cell.attr & attr::ALL_UNDERSCORE, attr::UNDERSCORE_3);
    }

    #[test]
    fn erasing_uses_the_current_background() {
        let mut engine = screen(6, 2);
        feed(&mut engine, b"\x1b[41m\x1b[2K");
        assert_eq!(engine.screen.grid.get(5, 0).bg, 1);
    }

    #[test]
    fn insert_mode_shifts_the_rest_of_the_row_along() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"abcd\x1b[1;2H\x1b[4hXY");
        assert_eq!(text(&engine, 0), "aXYbcd");
    }

    #[test]
    fn the_scroll_region_confines_a_linefeed() {
        let mut engine = screen(6, 4);
        feed(&mut engine, b"r0\r\nr1\r\nr2\r\nr3");
        feed(&mut engine, b"\x1b[2;3r\x1b[3;1H\n");
        assert_eq!(text(&engine, 0), "r0");
        assert_eq!(text(&engine, 1), "r2");
        assert_eq!(text(&engine, 2), "");
        assert_eq!(text(&engine, 3), "r3");
    }

    #[test]
    fn origin_mode_makes_addressing_relative_to_the_region() {
        let mut engine = screen(8, 6);
        feed(&mut engine, b"\x1b[3;5r\x1b[?6h\x1b[1;1Hx");
        assert_eq!(engine.screen.grid.get(0, 2).data.text(), "x");
    }

    #[test]
    fn rep_repeats_the_last_character_and_only_then() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"a\x1b[3b");
        assert_eq!(text(&engine, 0), "aaaa");

        let mut engine = screen(10, 2);
        feed(&mut engine, b"a\x1b[H\x1b[3b");
        assert_eq!(
            text(&engine, 0),
            "a",
            "a cursor move invalidates the repeat"
        );
    }

    #[test]
    fn tab_stops_can_be_set_cleared_and_walked_backwards() {
        let mut engine = screen(40, 2);
        feed(&mut engine, b"\x1b[1;5H\x1bH");
        assert!(engine.screen.tabs.contains(&4));
        feed(&mut engine, b"\x1b[1;1H\t");
        assert_eq!(engine.screen.cx, 4, "the new stop comes first");
        feed(&mut engine, b"\x1b[1;20H\x1b[Z");
        assert_eq!(engine.screen.cx, 16, "back to the previous default stop");
        feed(&mut engine, b"\x1b[3g\x1b[1;1H\t");
        assert_eq!(engine.screen.cx, 39, "with no stops left, the last column");
    }

    #[test]
    fn the_alternate_screen_switches_and_comes_back() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"primary\x1b[?1049h");
        assert!(engine.screen.saved_grid().is_some());
        assert_eq!(text(&engine, 0), "");
        feed(&mut engine, b"alt\x1b[?1049l");
        assert!(engine.screen.saved_grid().is_none());
        assert_eq!(text(&engine, 0), "primary");
        assert_eq!(engine.screen.cx, 7);
    }

    #[test]
    fn save_and_restore_carry_the_pen_with_the_cursor() {
        let mut engine = screen(10, 3);
        feed(&mut engine, b"\x1b[2;3H\x1b[31m\x1b7\x1b[1;1H\x1b[0m\x1b8X");
        assert_eq!((engine.screen.cx, engine.screen.cy), (3, 1));
        assert_eq!(engine.screen.grid.get(2, 1).fg, 1, "the colour came back");
    }

    #[test]
    fn the_line_drawing_set_marks_cells_rather_than_replacing_them() {
        let mut engine = screen(10, 2);
        feed(&mut engine, b"\x1b(0q\x1b(Bq");
        assert_eq!(
            engine.screen.grid.get(0, 0).attr & attr::CHARSET,
            attr::CHARSET
        );
        assert_eq!(engine.screen.grid.get(1, 0).attr & attr::CHARSET, 0);
    }

    #[test]
    fn scroll_up_and_down_move_the_region() {
        let mut engine = screen(6, 3);
        feed(&mut engine, b"a\r\nb\r\nc\x1b[2S");
        assert_eq!(text(&engine, 0), "c");
        assert_eq!(text(&engine, 1), "");

        let mut engine = screen(6, 3);
        feed(&mut engine, b"a\r\nb\r\nc\x1b[1T");
        assert_eq!(text(&engine, 0), "");
        assert_eq!(text(&engine, 1), "a");
    }

    #[test]
    fn a_reset_puts_everything_back() {
        let mut engine = screen(8, 3);
        feed(&mut engine, b"\x1b[31mtext\x1b[2;3r\x1b(0\x1bc");
        assert_eq!(text(&engine, 0), "");
        assert_eq!(engine.screen.cell.fg, colour::DEFAULT);
        assert_eq!((engine.screen.rupper, engine.screen.rlower), (0, 2));
        feed(&mut engine, b"q");
        assert_eq!(
            engine.screen.grid.get(0, 0).attr & attr::CHARSET,
            0,
            "the character set went back to ASCII too"
        );
    }

    #[test]
    fn a_colon_parameter_abandons_the_operation_outside_sgr() {
        // `input_split` types a position with a colon in it as `INPUT_STRING`,
        // `input_get` answers -1 for one, and the handler skips its operation
        // rather than acting on the part before the colon.
        let mut engine = screen(10, 5);
        feed(&mut engine, b"ab\x1b[4:E");
        assert_eq!((engine.screen.cx, engine.screen.cy), (2, 0));
        // The position a handler does not read is not the one that stops it.
        feed(&mut engine, b"\x1b[2;3:4H");
        assert_eq!((engine.screen.cx, engine.screen.cy), (2, 0));
        // SGR is the one operation that reads past the colon.
        feed(&mut engine, b"\x1b[4:3m");
        assert_eq!(
            engine.screen.cell.attr & attr::ALL_UNDERSCORE,
            attr::UNDERSCORE_3
        );
    }

    #[test]
    fn shell_integration_marks_the_rows_it_names() {
        let mut engine = screen(10, 3);
        feed(&mut engine, b"\x1b]133;A\x07$ ls\r\n\x1b]133;C\x07out");
        let grid = &engine.screen.grid;
        assert_eq!(
            grid.line(0).expect("row").flags & line_flag::START_PROMPT,
            line_flag::START_PROMPT
        );
        assert_eq!(
            grid.line(1).expect("row").flags & line_flag::START_OUTPUT,
            line_flag::START_OUTPUT
        );
    }
}

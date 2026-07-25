//! Safe wrapper over libghostty-vt: a [`Terminal`] you feed raw child-process
//! bytes and read back as a text grid, plus the Unicode width operations used
//! by that grid.
//!
//! This is the terminal-emulation core of the native (non-proxy) path. tmux's
//! own `input.c`/`screen.c` parse a pane's byte stream into a cell grid; here
//! libghostty-vt does that job, and [`Terminal::dump_plain`] reads the grid out
//! — which is both what a renderer needs and what herdr-style agent detection
//! inspects.
//!
//! Thread-safety: a `GhosttyTerminal` handle is not internally synchronized, so
//! [`Terminal`] is `Send` (it may move to a pane's reader thread) but **not**
//! `Sync`; callers serialize access (the pane holds it behind a mutex).

use std::fmt;
use std::mem;
use std::ptr;

mod ffi;

/// Return Ghostty's terminal-cell width for one Unicode codepoint.
///
/// The result is always 0, 1, or 2 and uses the same generated property table
/// as Ghostty's terminal grid. This function is total: values above U+10FFFF
/// have width one.
#[must_use]
pub fn codepoint_width(codepoint: u32) -> u8 {
    // SAFETY: the function accepts every `u32`, has no pointer arguments, and
    // is documented by libghostty-vt as pure and thread-safe.
    unsafe { ffi::ghostty_unicode_codepoint_width(codepoint) }
}

/// Measure the first Ghostty grapheme cluster in `codepoints`.
///
/// Returns `(consumed, width)`, where `consumed` is the number of input
/// codepoints belonging to the first cluster and `width` is its terminal-cell
/// width (0, 1, or 2). Empty input returns `(0, 0)`.
///
/// This applies the same segmentation and cluster-width rules as Ghostty with
/// DEC mode 2027 enabled. It is a complete-slice operation rather than a
/// streaming one: callers processing chunked input must retain a possible
/// trailing cluster until they know it is complete.
#[must_use]
pub fn grapheme_width(codepoints: &[u32]) -> (usize, u8) {
    let mut width = 0;
    let ptr = if codepoints.is_empty() {
        ptr::null()
    } else {
        codepoints.as_ptr()
    };
    // SAFETY: `ptr` is NULL only for empty input; otherwise it points to
    // `codepoints.len()` readable `u32`s. `width` is a valid out-pointer for
    // the duration of the call. The function does not retain either pointer.
    let consumed =
        unsafe { ffi::ghostty_unicode_grapheme_width(ptr, codepoints.len(), &mut width) };
    debug_assert!(consumed <= codepoints.len());
    (consumed, width)
}

/// A libghostty-vt error, wrapping the C `GhosttyResult` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(pub i32);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "libghostty-vt error (code {})", self.0)
    }
}

impl std::error::Error for Error {}

fn check(code: i32) -> Result<(), Error> {
    if code == ffi::GHOSTTY_SUCCESS {
        Ok(())
    } else {
        Err(Error(code))
    }
}

/// An owned terminal emulator instance: VT parser + screen grid + scrollback.
pub struct Terminal {
    raw: ffi::GhosttyTerminal,
}

/// Physical key identity understood by libghostty-vt's key encoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Key(i32);

impl Key {
    pub const UNIDENTIFIED: Key = Key(0);
    pub const BACKQUOTE: Key = Key(1);
    pub const BACKSLASH: Key = Key(2);
    pub const BRACKET_LEFT: Key = Key(3);
    pub const BRACKET_RIGHT: Key = Key(4);
    pub const COMMA: Key = Key(5);
    pub const DIGIT_0: Key = Key(6);
    pub const EQUAL: Key = Key(16);
    pub const A: Key = Key(20);
    pub const MINUS: Key = Key(46);
    pub const PERIOD: Key = Key(47);
    pub const QUOTE: Key = Key(48);
    pub const SEMICOLON: Key = Key(49);
    pub const SLASH: Key = Key(50);
    pub const BACKSPACE: Key = Key(53);
    pub const ENTER: Key = Key(58);
    pub const SPACE: Key = Key(63);
    pub const TAB: Key = Key(64);
    pub const DELETE: Key = Key(68);
    pub const END: Key = Key(69);
    pub const HOME: Key = Key(71);
    pub const INSERT: Key = Key(72);
    pub const PAGE_DOWN: Key = Key(73);
    pub const PAGE_UP: Key = Key(74);
    pub const ARROW_DOWN: Key = Key(75);
    pub const ARROW_LEFT: Key = Key(76);
    pub const ARROW_RIGHT: Key = Key(77);
    pub const ARROW_UP: Key = Key(78);
    pub const NUMPAD_0: Key = Key(80);
    pub const NUMPAD_ADD: Key = Key(90);
    pub const NUMPAD_DECIMAL: Key = Key(95);
    pub const NUMPAD_DIVIDE: Key = Key(96);
    pub const NUMPAD_ENTER: Key = Key(97);
    pub const NUMPAD_MULTIPLY: Key = Key(104);
    pub const NUMPAD_SUBTRACT: Key = Key(107);
    pub const ESCAPE: Key = Key(120);
    pub const F1: Key = Key(121);

    /// Map a printable US-layout ASCII key to its physical identity.
    pub fn from_ascii(ch: char) -> Key {
        match ch {
            '`' | '~' => Key::BACKQUOTE,
            '\\' | '|' => Key::BACKSLASH,
            '[' | '{' => Key::BRACKET_LEFT,
            ']' | '}' => Key::BRACKET_RIGHT,
            ',' | '<' => Key::COMMA,
            '0'..='9' => Key(Key::DIGIT_0.0 + (ch as i32 - '0' as i32)),
            '=' | '+' => Key::EQUAL,
            'a'..='z' => Key(Key::A.0 + (ch as i32 - 'a' as i32)),
            'A'..='Z' => Key(Key::A.0 + (ch as i32 - 'A' as i32)),
            '-' | '_' => Key::MINUS,
            '.' | '>' => Key::PERIOD,
            '\'' | '"' => Key::QUOTE,
            ';' | ':' => Key::SEMICOLON,
            '/' | '?' => Key::SLASH,
            ' ' => Key::SPACE,
            _ => Key::UNIDENTIFIED,
        }
    }

    pub fn function(number: u8) -> Option<Key> {
        (1..=12)
            .contains(&number)
            .then(|| Key(Key::F1.0 + i32::from(number - 1)))
    }

    pub fn numpad_digit(digit: char) -> Option<Key> {
        digit
            .to_digit(10)
            .map(|digit| Key(Key::NUMPAD_0.0 + digit as i32))
    }
}

/// One press event for libghostty-vt's terminal-state-aware encoder.
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent<'a> {
    pub key: Key,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub text: Option<&'a str>,
    pub unshifted_codepoint: Option<char>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum MouseAction {
    Press = 0,
    Release = 1,
    Motion = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 3,
    WheelUp = 4,
    WheelDown = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Eleven = 11,
}

/// One cell-addressed mouse event for Ghostty's terminal-state-aware encoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub button: Option<MouseButton>,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub column: u16,
    pub row: u16,
    /// Current aggregate button state, used by motion modes at viewport edges.
    pub any_button_pressed: bool,
}

/// An immutable copy of one Ghostty terminal cell.
///
/// `text` contains the complete grapheme cluster stored in the cell. Empty
/// cells have an empty string; this intentionally distinguishes them from a
/// literal U+0020 cell without claiming how an empty cell was produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridCellSnapshot {
    pub text: String,
    pub width: GridCellWidth,
    pub semantic: GridCellSemantic,
    pub hyperlink: Option<String>,
    pub hyperlink_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridCellSemantic {
    Output,
    Input,
    Prompt,
}

/// Ghostty's display-cell width/spacer classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridCellWidth {
    Narrow,
    Wide,
    SpacerTail,
    SpacerHead,
}

/// One physical row in an immutable terminal-grid snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridRowSnapshot {
    pub cells: Vec<GridCellSnapshot>,
    /// Whether this physical row soft-wraps into the following row.
    pub wrapped: bool,
}

/// An immutable copy of Ghostty's active screen, including scrollback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridSnapshot {
    pub cols: u16,
    pub viewport_rows: u16,
    pub scrollback_rows: usize,
    pub rows: Vec<GridRowSnapshot>,
}

// The handle is single-threaded but movable; access is externally serialized.
unsafe impl Send for Terminal {}

impl Terminal {
    // libghostty measures this in bytes (including the active screen), not in
    // rows. Ten megabytes gives ordinary desktop shell output substantially
    // more history than tmux's 2,000-row default without making an unbounded
    // row-count promise. Allocation is lazy, so this is a cap, not an up-front
    // 10 MB allocation.
    const DEFAULT_MAX_SCROLLBACK_BYTES: usize = 10_000_000;

    /// Create a `cols`×`rows` terminal with native-pane scrollback.
    pub fn new(cols: u16, rows: u16) -> Result<Terminal, Error> {
        let options = ffi::GhosttyTerminalOptions {
            cols: cols.max(1),
            rows: rows.max(1),
            max_scrollback: Self::DEFAULT_MAX_SCROLLBACK_BYTES,
        };
        let mut raw: ffi::GhosttyTerminal = ptr::null_mut();
        // SAFETY: `raw` is a valid out-pointer; NULL allocator = default.
        check(unsafe { ffi::ghostty_terminal_new(ptr::null(), &mut raw, options) })?;
        if raw.is_null() {
            return Err(Error(ffi::GHOSTTY_SUCCESS)); // succeeded but null: treat as error
        }
        let mut terminal = Terminal { raw };
        // tmux stores extended grapheme clusters in one display cell sequence.
        // Enable Ghostty's corresponding mode so emoji modifiers, ZWJ
        // sequences, and regional indicators retain tmux-compatible widths.
        terminal.write(b"\x1b[?2027h");
        Ok(terminal)
    }

    /// Feed raw VT bytes (a chunk of PTY output) through the parser. Never fails:
    /// malformed input is absorbed to keep state consistent (see header docs).
    pub fn write(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        // SAFETY: `self.raw` is a live handle; `data` is a valid slice.
        unsafe { ffi::ghostty_terminal_vt_write(self.raw, data.as_ptr(), data.len()) }
    }

    /// Encode a key press according to the terminal's current input modes.
    pub fn encode_key(&self, key: KeyEvent<'_>) -> Result<Vec<u8>, Error> {
        let mut encoder: ffi::GhosttyKeyEncoder = ptr::null_mut();
        check(unsafe { ffi::ghostty_key_encoder_new(ptr::null(), &mut encoder) })?;
        if encoder.is_null() {
            return Err(Error(ffi::GHOSTTY_SUCCESS));
        }

        let mut event: ffi::GhosttyKeyEvent = ptr::null_mut();
        let event_result = check(unsafe { ffi::ghostty_key_event_new(ptr::null(), &mut event) });
        if let Err(error) = event_result {
            unsafe { ffi::ghostty_key_encoder_free(encoder) };
            return Err(error);
        }
        if event.is_null() {
            unsafe { ffi::ghostty_key_encoder_free(encoder) };
            return Err(Error(ffi::GHOSTTY_SUCCESS));
        }

        unsafe {
            ffi::ghostty_key_encoder_setopt_from_terminal(encoder, self.raw);
            ffi::ghostty_key_event_set_action(event, 1);
            ffi::ghostty_key_event_set_key(event, key.key.0);
            let mods =
                u16::from(key.shift) | (u16::from(key.control) << 1) | (u16::from(key.alt) << 2);
            ffi::ghostty_key_event_set_mods(event, mods);
            ffi::ghostty_key_event_set_consumed_mods(event, 0);
            if let Some(text) = key.text {
                ffi::ghostty_key_event_set_utf8(event, text.as_ptr().cast(), text.len());
            }
            if let Some(codepoint) = key.unshifted_codepoint {
                ffi::ghostty_key_event_set_unshifted_codepoint(event, codepoint as u32);
            }
        }

        let mut buffer = vec![0u8; 128];
        let mut written = 0usize;
        let mut code = unsafe {
            ffi::ghostty_key_encoder_encode(
                encoder,
                event,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut written,
            )
        };
        if code == ffi::GHOSTTY_OUT_OF_SPACE {
            buffer.resize(written, 0);
            code = unsafe {
                ffi::ghostty_key_encoder_encode(
                    encoder,
                    event,
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut written,
                )
            };
        }
        unsafe {
            ffi::ghostty_key_event_free(event);
            ffi::ghostty_key_encoder_free(encoder);
        }
        check(code)?;
        buffer.truncate(written);
        Ok(buffer)
    }

    /// Whether the pane application has enabled any terminal mouse mode.
    pub fn mouse_tracking(&self) -> Result<bool, Error> {
        let mut enabled = false;
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_MOUSE_TRACKING,
                &mut enabled as *mut bool as *mut _,
            )
        })?;
        Ok(enabled)
    }

    /// Encode a cell-addressed mouse event according to the pane's current
    /// tracking mode and output format.
    pub fn encode_mouse(&self, mouse: MouseEvent) -> Result<Vec<u8>, Error> {
        let cols = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_COLS)?;
        let rows = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_ROWS)?;
        let mut encoder: ffi::GhosttyMouseEncoder = ptr::null_mut();
        check(unsafe { ffi::ghostty_mouse_encoder_new(ptr::null(), &mut encoder) })?;
        if encoder.is_null() {
            return Err(Error(ffi::GHOSTTY_SUCCESS));
        }

        let mut event: ffi::GhosttyMouseEvent = ptr::null_mut();
        if let Err(error) = check(unsafe { ffi::ghostty_mouse_event_new(ptr::null(), &mut event) })
        {
            unsafe { ffi::ghostty_mouse_encoder_free(encoder) };
            return Err(error);
        }
        if event.is_null() {
            unsafe { ffi::ghostty_mouse_encoder_free(encoder) };
            return Err(Error(ffi::GHOSTTY_SUCCESS));
        }

        let size = ffi::GhosttyMouseEncoderSize {
            size: mem::size_of::<ffi::GhosttyMouseEncoderSize>(),
            screen_width: u32::from(cols),
            screen_height: u32::from(rows),
            cell_width: 1,
            cell_height: 1,
            padding_top: 0,
            padding_bottom: 0,
            padding_right: 0,
            padding_left: 0,
        };
        unsafe {
            ffi::ghostty_mouse_encoder_setopt_from_terminal(encoder, self.raw);
            ffi::ghostty_mouse_encoder_setopt(
                encoder,
                ffi::GHOSTTY_MOUSE_ENCODER_OPT_SIZE,
                (&size as *const ffi::GhosttyMouseEncoderSize).cast(),
            );
            ffi::ghostty_mouse_encoder_setopt(
                encoder,
                ffi::GHOSTTY_MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED,
                (&mouse.any_button_pressed as *const bool).cast(),
            );
            ffi::ghostty_mouse_event_set_action(event, mouse.action as i32);
            if let Some(button) = mouse.button {
                ffi::ghostty_mouse_event_set_button(event, button as i32);
            } else {
                ffi::ghostty_mouse_event_clear_button(event);
            }
            let mods = u16::from(mouse.shift)
                | (u16::from(mouse.control) << 1)
                | (u16::from(mouse.alt) << 2);
            ffi::ghostty_mouse_event_set_mods(event, mods);
            ffi::ghostty_mouse_event_set_position(
                event,
                ffi::GhosttyMousePosition {
                    x: f32::from(mouse.column),
                    y: f32::from(mouse.row),
                },
            );
        }

        let mut buffer = vec![0u8; 128];
        let mut written = 0usize;
        let mut code = unsafe {
            ffi::ghostty_mouse_encoder_encode(
                encoder,
                event,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut written,
            )
        };
        if code == ffi::GHOSTTY_OUT_OF_SPACE {
            buffer.resize(written, 0);
            code = unsafe {
                ffi::ghostty_mouse_encoder_encode(
                    encoder,
                    event,
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut written,
                )
            };
        }
        unsafe {
            ffi::ghostty_mouse_event_free(event);
            ffi::ghostty_mouse_encoder_free(encoder);
        }
        check(code)?;
        buffer.truncate(written);
        Ok(buffer)
    }

    /// Resize the grid. Pixel cell size is irrelevant for text emulation, so we
    /// pass 1×1 (only image/size-report protocols care).
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Error> {
        // SAFETY: live handle; dimensions clamped to > 0.
        check(unsafe { ffi::ghostty_terminal_resize(self.raw, cols.max(1), rows.max(1), 1, 1) })
    }

    /// Return the cursor position in Ghostty's native 0-indexed coordinates.
    pub fn cursor_position(&self) -> Result<(u16, u16), Error> {
        let mut x = 0u16;
        let mut y = 0u16;
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_CURSOR_X,
                &mut x as *mut u16 as *mut _,
            )
        })?;
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_CURSOR_Y,
                &mut y as *mut u16 as *mut _,
            )
        })?;
        Ok((x, y))
    }

    /// Whether the cursor is visible (DEC private mode 25, DECTCEM).
    ///
    /// A full-screen TUI (e.g. an editor, or claude-code) typically *hides* the
    /// hardware cursor with `CSI ? 25 l` and paints its own cursor as a styled
    /// grid cell. The VT dump ([`dump_vt`]) carries the grid and a final cursor
    /// *position*, but not this *visibility* mode, so the compositor must query
    /// it separately and mirror it onto the client tty — otherwise the client's
    /// real cursor stays lit on top of the app's painted one (a double cursor).
    pub fn cursor_visible(&self) -> Result<bool, Error> {
        let mut visible = false;
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_CURSOR_VISIBLE,
                &mut visible as *mut bool as *mut _,
            )
        })?;
        Ok(visible)
    }

    /// The terminal title as set by escape sequences (OSC 0/2), or `None` when
    /// no title has been set.
    ///
    /// This is the signal herdr-style agent detection reads for agents (e.g.
    /// Codex) that report their live status in the window title rather than only
    /// on screen. The library hands back a *borrowed* pointer valid until the
    /// next `write`/reset; we copy it into an owned `String` while holding the
    /// terminal (callers serialize access behind the pane mutex), so no write
    /// can invalidate it mid-read.
    pub fn title(&self) -> Result<Option<String>, Error> {
        let mut string = ffi::GhosttyString {
            ptr: ptr::null(),
            len: 0,
        };
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_TITLE,
                &mut string as *mut ffi::GhosttyString as *mut _,
            )
        })?;
        if string.ptr.is_null() || string.len == 0 {
            return Ok(None);
        }
        // SAFETY: on success the library guarantees `ptr` addresses `len` valid
        // bytes; we copy them out before returning (and thus before any later
        // write could free the borrow).
        let bytes = unsafe { std::slice::from_raw_parts(string.ptr, string.len) };
        Ok(Some(String::from_utf8_lossy(bytes).into_owned()))
    }

    /// The number of scrollback (history) rows: total stored rows minus the
    /// visible viewport.
    ///
    /// The VT/plain formatters emit the *whole* screen — scrollback history
    /// first, then the visible viewport — so a consumer that wants only what a
    /// real terminal shows (the compositor, `capture-pane -p`) must skip this
    /// many leading rows. Without that, once output scrolls the pane the first
    /// screenful of the dump is the *oldest* history, and the client is left
    /// showing stale top-of-history text that `clear` cannot dislodge (it empties
    /// the viewport but not the retained history). See `report.md`.
    pub fn scrollback_rows(&self) -> Result<usize, Error> {
        let mut rows: usize = 0;
        check(unsafe {
            ffi::ghostty_terminal_get(
                self.raw,
                ffi::GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS,
                &mut rows as *mut usize as *mut _,
            )
        })?;
        Ok(rows)
    }

    /// Snapshot every physical cell and row in the active screen.
    ///
    /// This reads Ghostty's public grid-reference API synchronously, so all
    /// untracked references are consumed before this method returns and before
    /// the caller can mutate the terminal again.
    pub fn grid_snapshot(&self) -> Result<GridSnapshot, Error> {
        let cols = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_COLS)?;
        let viewport_rows = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_ROWS)?;
        let total_rows = self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_TOTAL_ROWS)?;
        let scrollback_rows = self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS)?;
        let mut rows = Vec::with_capacity(total_rows);

        for y in 0..total_rows {
            let y = u32::try_from(y).map_err(|_| Error(ffi::GHOSTTY_INVALID_VALUE))?;
            let mut cells = Vec::with_capacity(cols as usize);
            let mut wrapped = false;
            for x in 0..cols {
                let mut grid_ref = empty_grid_ref();
                check(unsafe {
                    ffi::ghostty_terminal_grid_ref(self.raw, screen_point(x, y), &mut grid_ref)
                })?;
                if x == 0 {
                    let mut row: ffi::GhosttyRow = 0;
                    check(unsafe { ffi::ghostty_grid_ref_row(&grid_ref, &mut row) })?;
                    check(unsafe {
                        ffi::ghostty_row_get(
                            row,
                            ffi::GHOSTTY_ROW_DATA_WRAP,
                            &mut wrapped as *mut bool as *mut _,
                        )
                    })?;
                }

                let mut cell: ffi::GhosttyCell = 0;
                check(unsafe { ffi::ghostty_grid_ref_cell(&grid_ref, &mut cell) })?;
                let mut wide = ffi::GHOSTTY_CELL_WIDE_NARROW;
                check(unsafe {
                    ffi::ghostty_cell_get(
                        cell,
                        ffi::GHOSTTY_CELL_DATA_WIDE,
                        &mut wide as *mut i32 as *mut _,
                    )
                })?;
                let width = match wide {
                    ffi::GHOSTTY_CELL_WIDE_NARROW => GridCellWidth::Narrow,
                    ffi::GHOSTTY_CELL_WIDE_WIDE => GridCellWidth::Wide,
                    ffi::GHOSTTY_CELL_WIDE_SPACER_TAIL => GridCellWidth::SpacerTail,
                    ffi::GHOSTTY_CELL_WIDE_SPACER_HEAD => GridCellWidth::SpacerHead,
                    _ => return Err(Error(ffi::GHOSTTY_INVALID_VALUE)),
                };
                let text = self.grid_ref_text(&grid_ref)?;
                let mut semantic = ffi::GHOSTTY_CELL_SEMANTIC_OUTPUT;
                check(unsafe {
                    ffi::ghostty_cell_get(
                        cell,
                        ffi::GHOSTTY_CELL_DATA_SEMANTIC_CONTENT,
                        &mut semantic as *mut i32 as *mut _,
                    )
                })?;
                let semantic = match semantic {
                    ffi::GHOSTTY_CELL_SEMANTIC_OUTPUT => GridCellSemantic::Output,
                    ffi::GHOSTTY_CELL_SEMANTIC_INPUT => GridCellSemantic::Input,
                    ffi::GHOSTTY_CELL_SEMANTIC_PROMPT => GridCellSemantic::Prompt,
                    _ => return Err(Error(ffi::GHOSTTY_INVALID_VALUE)),
                };
                let hyperlink =
                    self.grid_ref_hyperlink(&grid_ref, ffi::ghostty_grid_ref_hyperlink_uri)?;
                let hyperlink_id =
                    self.grid_ref_hyperlink(&grid_ref, ffi::ghostty_grid_ref_hyperlink_id)?;
                cells.push(GridCellSnapshot {
                    text,
                    width,
                    semantic,
                    hyperlink,
                    hyperlink_id,
                });
            }
            rows.push(GridRowSnapshot { cells, wrapped });
        }

        Ok(GridSnapshot {
            cols,
            viewport_rows,
            scrollback_rows,
            rows,
        })
    }

    fn grid_ref_text(&self, grid_ref: &ffi::GhosttyGridRef) -> Result<String, Error> {
        let mut needed = 0;
        let code =
            unsafe { ffi::ghostty_grid_ref_graphemes(grid_ref, ptr::null_mut(), 0, &mut needed) };
        if needed == 0 {
            check(code)?;
            return Ok(String::new());
        }
        if code != ffi::GHOSTTY_OUT_OF_SPACE {
            check(code)?;
        }
        let mut codepoints = vec![0u32; needed];
        let mut written = 0;
        check(unsafe {
            ffi::ghostty_grid_ref_graphemes(
                grid_ref,
                codepoints.as_mut_ptr(),
                codepoints.len(),
                &mut written,
            )
        })?;
        codepoints.truncate(written);
        let mut text = String::new();
        for codepoint in codepoints {
            text.push(char::from_u32(codepoint).ok_or(Error(ffi::GHOSTTY_INVALID_VALUE))?);
        }
        Ok(text)
    }

    fn grid_ref_hyperlink(
        &self,
        grid_ref: &ffi::GhosttyGridRef,
        getter: unsafe extern "C" fn(*const ffi::GhosttyGridRef, *mut u8, usize, *mut usize) -> i32,
    ) -> Result<Option<String>, Error> {
        let mut needed = 0;
        let code = unsafe { getter(grid_ref, ptr::null_mut(), 0, &mut needed) };
        if needed == 0 {
            check(code)?;
            return Ok(None);
        }
        if code != ffi::GHOSTTY_OUT_OF_SPACE {
            check(code)?;
        }
        let mut bytes = vec![0_u8; needed];
        let mut written = 0;
        check(unsafe { getter(grid_ref, bytes.as_mut_ptr(), bytes.len(), &mut written) })?;
        bytes.truncate(written);
        Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn get_u16(&self, data: i32) -> Result<u16, Error> {
        let mut value = 0u16;
        check(unsafe {
            ffi::ghostty_terminal_get(self.raw, data, &mut value as *mut u16 as *mut _)
        })?;
        Ok(value)
    }

    fn get_usize(&self, data: i32) -> Result<usize, Error> {
        let mut value = 0usize;
        check(unsafe {
            ffi::ghostty_terminal_get(self.raw, data, &mut value as *mut usize as *mut _)
        })?;
        Ok(value)
    }

    /// Render the active screen as plain text (no escapes), trailing whitespace
    /// trimmed. This is the "read the grid" primitive.
    pub fn dump_plain(&self) -> Result<String, Error> {
        self.dump_plain_with(false)
    }

    /// Render the active screen as plain text with soft-wrapped rows rejoined
    /// into their logical lines. A row the terminal wrapped only because it hit
    /// the right margin is emitted as one line here, rather than split at the
    /// (invisible) wrap point — what a reader wants when reading back output.
    pub fn dump_plain_unwrapped(&self) -> Result<String, Error> {
        self.dump_plain_with(true)
    }

    fn dump_plain_with(&self, unwrap: bool) -> Result<String, Error> {
        let options = plain_formatter_options(unwrap);

        let mut fmt: ffi::GhosttyFormatter = ptr::null_mut();
        // SAFETY: valid out-pointer + live terminal; NULL allocator = default.
        check(unsafe {
            ffi::ghostty_formatter_terminal_new(ptr::null(), &mut fmt, self.raw, options)
        })?;

        let result = self.format_to_string(fmt);
        // SAFETY: `fmt` was created above and is freed exactly once here.
        unsafe { ffi::ghostty_formatter_free(fmt) };
        result
    }

    /// Render the active screen as VT escape sequences suitable for writing
    /// directly to a client tty. Includes cursor position, SGR styles, and
    /// other screen state needed for faithful reproduction.
    ///
    /// This is the compositor primitive for `attach-session`: the server owns
    /// the pane's `Terminal` (the grid), and on each repaint it formats the
    /// grid as VT and writes it to the client's tty fd.
    pub fn dump_vt(&self) -> Result<Vec<u8>, Error> {
        self.dump_vt_selection(None)
    }

    /// Render a contiguous range of full-screen rows as VT without formatting
    /// rows outside that range. `start` is zero-based from oldest history.
    pub fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> Result<Vec<u8>, Error> {
        if rows == 0 || cols == 0 {
            return Ok(Vec::new());
        }
        let end = start.saturating_add(rows - 1);
        let start_y = u32::try_from(start).map_err(|_| Error(ffi::GHOSTTY_INVALID_VALUE))?;
        let end_y = u32::try_from(end).map_err(|_| Error(ffi::GHOSTTY_INVALID_VALUE))?;
        let mut start_ref = empty_grid_ref();
        let mut end_ref = empty_grid_ref();
        check(unsafe {
            ffi::ghostty_terminal_grid_ref(self.raw, screen_point(0, start_y), &mut start_ref)
        })?;
        check(unsafe {
            ffi::ghostty_terminal_grid_ref(self.raw, screen_point(cols - 1, end_y), &mut end_ref)
        })?;
        let selection = ffi::GhosttySelection {
            size: mem::size_of::<ffi::GhosttySelection>(),
            start: start_ref,
            end: end_ref,
            rectangle: false,
        };
        self.dump_vt_selection(Some(&selection))
    }

    fn dump_vt_selection(
        &self,
        selection: Option<&ffi::GhosttySelection>,
    ) -> Result<Vec<u8>, Error> {
        let mut options = vt_formatter_options();
        options.selection = selection
            .map(|value| value as *const ffi::GhosttySelection as *const _)
            .unwrap_or(ptr::null());

        let mut fmt: ffi::GhosttyFormatter = ptr::null_mut();
        check(unsafe {
            ffi::ghostty_formatter_terminal_new(ptr::null(), &mut fmt, self.raw, options)
        })?;

        let result = self.format_to_bytes(fmt);
        unsafe { ffi::ghostty_formatter_free(fmt) };
        result
    }

    /// Two-call format: query the required size, then fill a right-sized buffer.
    fn format_to_string(&self, fmt: ffi::GhosttyFormatter) -> Result<String, Error> {
        // First call with a NULL buffer reports the needed size via `needed`
        // (returning OUT_OF_SPACE, or SUCCESS when nothing is needed).
        let mut needed: usize = 0;
        // SAFETY: NULL buf + 0 len is the documented size-query form.
        unsafe { ffi::ghostty_formatter_format_buf(fmt, ptr::null_mut(), 0, &mut needed) };
        if needed == 0 {
            return Ok(String::new());
        }

        let mut buf = vec![0u8; needed];
        let mut written: usize = 0;
        // SAFETY: `buf` has `needed` bytes; `written` is a valid out-pointer.
        let code = unsafe {
            ffi::ghostty_formatter_format_buf(fmt, buf.as_mut_ptr(), buf.len(), &mut written)
        };
        check(code)?;
        buf.truncate(written);
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    fn format_to_bytes(&self, fmt: ffi::GhosttyFormatter) -> Result<Vec<u8>, Error> {
        let mut needed: usize = 0;
        unsafe { ffi::ghostty_formatter_format_buf(fmt, ptr::null_mut(), 0, &mut needed) };
        if needed == 0 {
            return Ok(Vec::new());
        }

        let mut buf = vec![0u8; needed];
        let mut written: usize = 0;
        let code = unsafe {
            ffi::ghostty_formatter_format_buf(fmt, buf.as_mut_ptr(), buf.len(), &mut written)
        };
        check(code)?;
        buf.truncate(written);
        Ok(buf)
    }
}

fn empty_grid_ref() -> ffi::GhosttyGridRef {
    ffi::GhosttyGridRef {
        size: mem::size_of::<ffi::GhosttyGridRef>(),
        node: ptr::null_mut(),
        x: 0,
        y: 0,
    }
}

fn screen_point(x: u16, y: u32) -> ffi::GhosttyPoint {
    ffi::GhosttyPoint {
        tag: ffi::GHOSTTY_POINT_TAG_SCREEN,
        value: ffi::GhosttyPointValue {
            coordinate: ffi::GhosttyPointCoordinate { x, y },
        },
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // SAFETY: freeing a handle we own; safe to call on NULL too.
        unsafe { ffi::ghostty_terminal_free(self.raw) };
    }
}

/// Plain-text formatter options: all "extra" state off, trailing whitespace
/// trimmed. The `size` fields let the library detect our struct version. When
/// `unwrap` is set, rows split only by a right-margin soft wrap are rejoined.
fn plain_formatter_options(unwrap: bool) -> ffi::GhosttyFormatterTerminalOptions {
    ffi::GhosttyFormatterTerminalOptions {
        size: mem::size_of::<ffi::GhosttyFormatterTerminalOptions>(),
        emit: ffi::GHOSTTY_FORMATTER_FORMAT_PLAIN,
        unwrap,
        trim: true,
        extra: ffi::GhosttyFormatterTerminalExtra {
            size: mem::size_of::<ffi::GhosttyFormatterTerminalExtra>(),
            palette: false,
            modes: false,
            scrolling_region: false,
            tabstops: false,
            pwd: false,
            keyboard: false,
            screen: ffi::GhosttyFormatterScreenExtra {
                size: mem::size_of::<ffi::GhosttyFormatterScreenExtra>(),
                cursor: false,
                style: false,
                hyperlink: false,
                protection: false,
                kitty_keyboard: false,
                charsets: false,
            },
        },
        selection: ptr::null(),
    }
}

/// VT formatter options for the compositor: emit full VT sequences including
/// cursor position, SGR styles, and other screen state needed to reproduce the
/// pane on a client tty. This is what `attach-session` writes to the client's
/// terminal fd.
fn vt_formatter_options() -> ffi::GhosttyFormatterTerminalOptions {
    ffi::GhosttyFormatterTerminalOptions {
        size: mem::size_of::<ffi::GhosttyFormatterTerminalOptions>(),
        emit: ffi::GHOSTTY_FORMATTER_FORMAT_VT,
        unwrap: false,
        trim: false,
        extra: ffi::GhosttyFormatterTerminalExtra {
            size: mem::size_of::<ffi::GhosttyFormatterTerminalExtra>(),
            palette: false,
            modes: false,
            scrolling_region: false,
            tabstops: false,
            pwd: false,
            keyboard: false,
            screen: ffi::GhosttyFormatterScreenExtra {
                size: mem::size_of::<ffi::GhosttyFormatterScreenExtra>(),
                cursor: true,
                style: true,
                hyperlink: true,
                protection: false,
                kitty_keyboard: false,
                charsets: false,
            },
        },
        selection: ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_width_uses_ghosttys_terminal_table() {
        assert_eq!(codepoint_width('a' as u32), 1);
        assert_eq!(codepoint_width('界' as u32), 2);
        assert_eq!(codepoint_width(0x0301), 0); // combining acute accent
        assert_eq!(codepoint_width('ㄱ' as u32), 2);
        assert_eq!(codepoint_width(0x1161), 0); // conjoining Hangul Jamo vowel
        assert_eq!(codepoint_width(0x110000), 1); // total for invalid codepoints
    }

    #[test]
    fn grapheme_width_measures_only_the_first_cluster() {
        let woman_technologist = [0x1F469, 0x200D, 0x1F4BB, 'x' as u32];
        assert_eq!(grapheme_width(&woman_technologist), (3, 2));
        assert_eq!(grapheme_width(&woman_technologist[3..]), (1, 1));
    }

    #[test]
    fn grapheme_width_handles_empty_and_zero_width_clusters() {
        assert_eq!(grapheme_width(&[]), (0, 0));
        assert_eq!(grapheme_width(&[0x0301, 0x0302]), (2, 0));
    }

    #[test]
    fn key_encoder_tracks_terminal_cursor_mode() {
        let mut term = Terminal::new(20, 5).expect("new terminal");
        let up = KeyEvent {
            key: Key::ARROW_UP,
            shift: false,
            control: false,
            alt: false,
            text: None,
            unshifted_codepoint: None,
        };
        assert_eq!(term.encode_key(up).unwrap(), b"\x1b[A");
        term.write(b"\x1b[?1h");
        assert_eq!(term.encode_key(up).unwrap(), b"\x1bOA");
    }

    #[test]
    fn mouse_encoder_tracks_terminal_mode_and_sgr_format() {
        let mut term = Terminal::new(20, 5).expect("new terminal");
        let press = MouseEvent {
            action: MouseAction::Press,
            button: Some(MouseButton::Left),
            shift: false,
            control: false,
            alt: false,
            column: 2,
            row: 3,
            any_button_pressed: true,
        };
        assert!(!term.mouse_tracking().expect("mouse mode"));
        assert_eq!(term.encode_mouse(press).expect("disabled encoding"), b"");

        term.write(b"\x1b[?1000h\x1b[?1006h");
        assert!(term.mouse_tracking().expect("mouse mode"));
        assert_eq!(
            term.encode_mouse(press).expect("SGR press"),
            b"\x1b[<0;3;4M"
        );
        assert_eq!(
            term.encode_mouse(MouseEvent {
                action: MouseAction::Release,
                any_button_pressed: false,
                ..press
            })
            .expect("SGR release"),
            b"\x1b[<0;3;4m"
        );
    }

    #[test]
    fn mouse_encoder_preserves_modifiers_and_wheel_buttons() {
        let mut term = Terminal::new(20, 5).expect("new terminal");
        term.write(b"\x1b[?1002h\x1b[?1006h");
        let event = MouseEvent {
            action: MouseAction::Press,
            button: Some(MouseButton::WheelUp),
            shift: true,
            control: true,
            alt: false,
            column: 0,
            row: 0,
            any_button_pressed: false,
        };
        assert_eq!(
            term.encode_mouse(event).expect("modified wheel"),
            b"\x1b[<84;1;1M"
        );
    }

    #[test]
    fn plain_text_round_trips_through_the_grid() {
        let mut term = Terminal::new(20, 5).expect("new terminal");
        term.write(b"hello world");
        let dump = term.dump_plain().expect("dump");
        assert!(
            dump.contains("hello world"),
            "grid should show written text, got {dump:?}"
        );
    }

    #[test]
    fn tab_spaces_and_cursor_gap_render_as_the_same_blanks() {
        for input in [b"\tX".as_slice(), b"        X", b"\x1b[9GX"] {
            let mut term = Terminal::new(20, 3).expect("new terminal");
            term.write(input);
            assert_eq!(
                term.dump_plain().expect("dump"),
                "        X",
                "input {input:?}"
            );
        }
    }

    #[test]
    fn grid_snapshot_distinguishes_spaces_from_empty_gaps() {
        let cases = [
            (b"\tX".as_slice(), false),
            (b"        X".as_slice(), true),
            (b"\x1b[9GX".as_slice(), false),
        ];
        for (input, written_spaces) in cases {
            let mut term = Terminal::new(12, 2).expect("new terminal");
            term.write(input);
            let snapshot = term.grid_snapshot().expect("snapshot");
            assert_eq!(snapshot.cols, 12);
            assert_eq!(snapshot.viewport_rows, 2);
            assert_eq!(snapshot.rows.len(), 2);
            for cell in &snapshot.rows[0].cells[..8] {
                assert_eq!(cell.text == " ", written_spaces);
            }
            assert_eq!(snapshot.rows[0].cells[8].text, "X");
        }
    }

    #[test]
    fn grid_snapshot_retains_graphemes_width_and_soft_wraps() {
        let mut term = Terminal::new(4, 3).expect("new terminal");
        term.write("e\u{301}界xy".as_bytes());
        let snapshot = term.grid_snapshot().expect("snapshot");
        assert_eq!(snapshot.rows[0].cells[0].text, "e\u{301}");
        assert_eq!(snapshot.rows[0].cells[0].width, GridCellWidth::Narrow);
        assert_eq!(snapshot.rows[0].cells[1].text, "界");
        assert_eq!(snapshot.rows[0].cells[1].width, GridCellWidth::Wide);
        assert_eq!(snapshot.rows[0].cells[2].width, GridCellWidth::SpacerTail);
        assert_eq!(snapshot.rows[0].cells[3].text, "x");
        assert!(snapshot.rows[0].wrapped);
        assert_eq!(snapshot.rows[1].cells[0].text, "y");
    }

    #[test]
    fn csi_cursor_move_and_overwrite() {
        let mut term = Terminal::new(20, 3).expect("new terminal");
        // Write "AAAAA", carriage-return to col 0, overwrite first two with "BB".
        term.write(b"AAAAA\rBB");
        let dump = term.dump_plain().expect("dump");
        assert!(dump.contains("BBAAA"), "CR + overwrite, got {dump:?}");
    }

    #[test]
    fn cursor_position_reports_inner_grid_coordinates() {
        let mut term = Terminal::new(20, 3).expect("new terminal");
        term.write(b"abc\r\nxy");
        assert_eq!(term.cursor_position().expect("cursor position"), (2, 1));
    }

    #[test]
    fn newlines_produce_multiple_rows() {
        let mut term = Terminal::new(20, 4).expect("new terminal");
        term.write(b"line1\r\nline2\r\nline3");
        let dump = term.dump_plain().expect("dump");
        for expected in ["line1", "line2", "line3"] {
            assert!(dump.contains(expected), "missing {expected:?} in {dump:?}");
        }
    }

    #[test]
    fn sgr_color_codes_are_absorbed_not_shown() {
        let mut term = Terminal::new(20, 2).expect("new terminal");
        // Red "X" then reset — plain format must not leak the escape bytes.
        term.write(b"\x1b[31mX\x1b[0m");
        let dump = term.dump_plain().expect("dump");
        assert!(dump.contains('X'), "text present, got {dump:?}");
        assert!(
            !dump.contains('\x1b'),
            "no raw escapes in plain dump: {dump:?}"
        );
    }

    #[test]
    fn vt_dump_contains_text_and_escapes() {
        let mut term = Terminal::new(20, 3).expect("new terminal");
        term.write(b"hello");
        let vt = term.dump_vt().expect("vt dump");
        // VT output should contain the text and at least one escape sequence
        // (cursor positioning or SGR). It must not be empty.
        assert!(!vt.is_empty(), "vt dump should not be empty");
        assert!(
            vt.windows(5).any(|w| w == b"hello"),
            "vt dump should contain text, got {:?}",
            String::from_utf8_lossy(&vt)
        );
        // The VT formatter emits CUP / ED or at least ESC sequences for cursor.
        assert!(
            vt.contains(&0x1b),
            "vt dump should contain escape bytes, got {:?}",
            String::from_utf8_lossy(&vt)
        );
    }

    #[test]
    fn cursor_visible_tracks_dectcem() {
        let mut term = Terminal::new(20, 3).expect("new terminal");
        // Default: the cursor is visible.
        assert!(term.cursor_visible().expect("query"), "default is visible");
        // A TUI hides the cursor with DECTCEM reset (CSI ? 25 l)…
        term.write(b"\x1b[?25l");
        assert!(!term.cursor_visible().expect("query"), "hidden after ?25l");
        // …and shows it again with DECTCEM set (CSI ? 25 h).
        term.write(b"\x1b[?25h");
        assert!(term.cursor_visible().expect("query"), "visible after ?25h");
    }

    #[test]
    fn vt_dump_omits_cursor_visibility_mode() {
        // Guard the premise of the compositor fix: dump_vt carries the cursor
        // *position* but not its *visibility*, so the compositor must query
        // `cursor_visible()` and emit DECTCEM itself. If ghostty ever starts
        // emitting ?25 here, revisit the double-cursor handling in the compositor.
        let mut term = Terminal::new(20, 3).expect("new terminal");
        term.write(b"\x1b[?25l");
        term.write(b"hi");
        let vt = term.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        assert!(
            !s.contains("\x1b[?25l") && !s.contains("\x1b[?25h"),
            "dump_vt must not carry cursor visibility, got {s:?}"
        );
    }

    #[test]
    fn unwrapped_dump_rejoins_soft_wrapped_rows() {
        // Write more than one row's worth of text with no newline: the terminal
        // soft-wraps it across two grid rows at the right margin.
        let mut term = Terminal::new(10, 4).expect("new terminal");
        term.write(b"abcdefghijKLMNOP");
        // The plain dump preserves the wrap as two rows…
        let wrapped = term.dump_plain().expect("dump");
        assert_eq!(wrapped.lines().count(), 2, "wrapped: {wrapped:?}");
        // …while the unwrapped dump rejoins them into one logical line.
        let unwrapped = term.dump_plain_unwrapped().expect("dump");
        assert_eq!(
            unwrapped.trim_end(),
            "abcdefghijKLMNOP",
            "unwrapped should be one line, got {unwrapped:?}"
        );
    }

    #[test]
    fn title_tracks_osc_0_and_2_and_starts_unset() {
        let mut term = Terminal::new(20, 3).expect("new terminal");
        // No title set yet.
        assert_eq!(term.title().expect("title"), None);
        // OSC 2 (set window title), BEL-terminated.
        term.write(b"\x1b]2;Working (3s)\x07");
        assert_eq!(
            term.title().expect("title").as_deref(),
            Some("Working (3s)")
        );
        // OSC 0 (set icon+window title), ST-terminated, replaces it.
        term.write(b"\x1b]0;Action Required\x1b\\");
        assert_eq!(
            term.title().expect("title").as_deref(),
            Some("Action Required")
        );
    }

    #[test]
    fn scrollback_rows_counts_history_beyond_the_viewport() {
        // A 4-row screen. Nothing scrolled yet → no scrollback.
        let mut term = Terminal::new(80, 4).expect("new terminal");
        term.write(b"A\r\nB");
        assert_eq!(term.scrollback_rows().expect("count"), 0);

        // Write 8 lines into the 4-row screen: 4 rows scroll into history.
        let mut term = Terminal::new(80, 4).expect("new terminal");
        term.write(b"A\r\nB\r\nC\r\nD\r\nE\r\nF\r\nG\r\nH");
        assert_eq!(
            term.scrollback_rows().expect("count"),
            4,
            "four rows (A–D) scrolled above the 4-row viewport"
        );

        // The dump carries history *plus* the viewport, oldest first: the count
        // is exactly how many leading rows a viewport-only consumer must skip.
        let vt = term.dump_vt().expect("vt");
        let text = String::from_utf8_lossy(&vt);
        let rows: Vec<&str> = text.split('\n').collect();
        assert!(
            rows[0].starts_with('A'),
            "dump starts at oldest history row"
        );
        assert!(
            rows[term.scrollback_rows().unwrap()].starts_with('E'),
            "skipping scrollback_rows lands on the first visible row (E)"
        );
    }

    /// Representative-workload conformance: an ordinary 80-column desktop
    /// shell should retain more than twice tmux's default 2,000-row history.
    /// This deliberately does not promise a fixed row count for arbitrary
    /// widths, styles, Unicode content, or wrapping: libghostty's cap is bytes.
    #[test]
    fn default_scrollback_retains_4000_ordinary_shell_lines() {
        const TMUX_DEFAULT_HISTORY_ROWS: usize = 2_000;
        const REQUIRED_HISTORY_ROWS: usize = TMUX_DEFAULT_HISTORY_ROWS * 2;
        const VIEWPORT_ROWS: usize = 24;

        let mut term = Terminal::new(80, VIEWPORT_ROWS as u16).expect("new terminal");
        for n in 0..(REQUIRED_HISTORY_ROWS + VIEWPORT_ROWS) {
            term.write(format!("SCROLLLINE_{n:04}\r\n").as_bytes());
        }

        let retained = term.scrollback_rows().expect("scrollback row count");
        assert!(
            retained >= REQUIRED_HISTORY_ROWS,
            "native retained {retained} representative rows; expected at least \
             {REQUIRED_HISTORY_ROWS} for ordinary 80-column shell output"
        );
        let dump = term.dump_plain().expect("history dump");
        assert!(
            dump.contains("SCROLLLINE_0000"),
            "the oldest row in the representative 4,000-line workload was discarded"
        );
    }

    #[test]
    fn vt_dump_preserves_colors() {
        let mut term = Terminal::new(20, 2).expect("new terminal");
        term.write(b"\x1b[31mRED\x1b[0m");
        let vt = term.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        // VT should preserve the color via SGR, unlike plain.
        assert!(s.contains("RED"), "vt should contain text, got {s:?}");
        assert!(
            s.contains("\x1b["),
            "vt should contain SGR/CUP escapes, got {s:?}"
        );
    }
}

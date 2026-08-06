//! The libghostty-vt backend for the emulation seam.
//!
//! [`GhosttyScreen`] is a safe wrapper over the `ghostty-sys` bindings that
//! implements [`VtScreen`] and [`InputEncoder`]. Everything specific to that
//! library lives here — the FFI, the two-call buffer protocol, the option
//! structs — so nothing above the seam names it.
//!
//! Where the library does not expose what the daemon needs, the reconstruction
//! is this backend's problem and not the server's. That is the whole point of
//! putting the seam here rather than at the library's own surface.
//!
//! This is no longer the shipped backend — [`crate::vt::PaneScreen`] names the
//! in-house engine — and the whole module is behind the `ghostty` feature. It
//! is kept as the alternate implementation the seam was carved to allow, and
//! as the other side of [`crate::vt::differential`], which is its only caller.

// The differential harness is test-only, so in a non-test build with this
// feature on nothing constructs the backend.
#![allow(dead_code)]

use std::io;
use std::mem;
use std::ptr;

use ghostty_sys::ffi;

use super::input::{InputEncoder, KeyEvent, MouseAction, MouseButton, MouseEvent};
use super::parser::Token;
use super::screen::{
    CaptureExtent, CellSemantic, CellWidth, Grid, GridCell, GridDims, GridRow, RowFlags,
    ScreenOptions, VtScreen,
};

/// A libghostty-vt error, wrapping the C `GhosttyResult` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Error(pub(crate) i32);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "libghostty-vt error (code {})", self.0)
    }
}

impl std::error::Error for Error {}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        io::Error::other(error)
    }
}

fn check(code: i32) -> Result<(), Error> {
    if code == ffi::GHOSTTY_SUCCESS {
        Ok(())
    } else {
        Err(Error(code))
    }
}

/// An owned terminal emulator instance: VT parser + screen grid + scrollback.
pub(crate) struct GhosttyScreen {
    raw: ffi::GhosttyTerminal,
}

// The handle is single-threaded but movable; access is externally serialized.
unsafe impl Send for GhosttyScreen {}

impl GhosttyScreen {
    // libghostty measures this in bytes (including the active screen), not in
    // rows. Ten megabytes gives ordinary desktop shell output substantially
    // more history than tmux's 2,000-row default without making an unbounded
    // row-count promise. Allocation is lazy, so this is a cap, not an up-front
    // 10 MB allocation.
    const DEFAULT_MAX_SCROLLBACK_BYTES: usize = 10_000_000;

    /// Create a `cols`×`rows` terminal with native-pane scrollback.
    pub(crate) fn new(cols: u16, rows: u16) -> Result<GhosttyScreen, Error> {
        let mut raw: ffi::GhosttyTerminal = ptr::null_mut();
        // SAFETY: `raw` is a valid out-pointer; NULL allocator = default.
        check(unsafe {
            ffi::ghostty_terminal_new(ptr::null(), &mut raw, cols.max(1), rows.max(1))
        })?;
        if raw.is_null() {
            return Err(Error(ffi::GHOSTTY_SUCCESS)); // succeeded but null: treat as error
        }
        let mut screen = GhosttyScreen { raw };
        let max_scrollback = Self::DEFAULT_MAX_SCROLLBACK_BYTES;
        // SAFETY: live handle; the byte limit option takes a `size_t*`.
        check(unsafe {
            ffi::ghostty_terminal_set(
                raw,
                ffi::GHOSTTY_TERMINAL_OPT_SCROLLBACK_MAX_BYTES,
                (&max_scrollback as *const usize).cast(),
            )
        })?;
        // tmux stores extended grapheme clusters in one display cell sequence.
        // Enable Ghostty's corresponding mode so emoji modifiers, ZWJ
        // sequences, and regional indicators retain tmux-compatible widths.
        screen.write(b"\x1b[?2027h");
        Ok(screen)
    }

    /// Feed raw VT bytes (a chunk of PTY output) through the parser. Never fails:
    /// malformed input is absorbed to keep state consistent (see header docs).
    pub(crate) fn write(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        // SAFETY: `self.raw` is a live handle; `data` is a valid slice.
        unsafe { ffi::ghostty_terminal_vt_write(self.raw, data.as_ptr(), data.len()) }
    }

    /// Encode a key press according to the terminal's current input modes.
    fn encode_key(&self, key: KeyEvent<'_>) -> Result<Vec<u8>, Error> {
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
            ffi::ghostty_key_event_set_key(event, key.key.code());
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

    /// Encode a cell-addressed mouse event according to the pane's current
    /// tracking mode and output format.
    fn encode_mouse(&self, mouse: MouseEvent) -> Result<Vec<u8>, Error> {
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
            ffi::ghostty_mouse_event_set_action(event, ffi_action(mouse.action));
            if let Some(button) = mouse.button {
                ffi::ghostty_mouse_event_set_button(event, ffi_button(button));
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
    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Error> {
        // SAFETY: live handle; dimensions clamped to > 0.
        check(unsafe { ffi::ghostty_terminal_resize(self.raw, cols.max(1), rows.max(1), 1, 1) })
    }

    /// Return the cursor position in Ghostty's native 0-indexed coordinates.
    fn cursor_position(&self) -> Result<(u16, u16), Error> {
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
    fn cursor_visible(&self) -> Result<bool, Error> {
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
    fn scrollback_rows(&self) -> Result<usize, Error> {
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

    /// Row geometry of the active screen: a handful of scalar reads, no cell
    /// walk. Callers that only need a row range use this to size the range
    /// before paying for [`Self::grid_snapshot_range`].
    fn grid_dims(&self) -> Result<GridDims, Error> {
        Ok(GridDims {
            cols: self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_COLS)?,
            viewport_rows: self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_ROWS)?,
            scrollback_rows: self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS)?,
            total_rows: self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_TOTAL_ROWS)?,
        })
    }

    /// Snapshot every physical cell and row in the active screen.
    ///
    /// This reads Ghostty's public grid-reference API synchronously, so all
    /// untracked references are consumed before this method returns and before
    /// the caller can mutate the terminal again.
    fn grid_snapshot(&self) -> Result<Grid, Error> {
        let total_rows = self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_TOTAL_ROWS)?;
        self.grid_snapshot_range(0, total_rows)
    }

    /// Snapshot only physical rows `[start, start + count)`, clamped to the
    /// grid. The per-cell walk is by far the dominant cost of a snapshot, so
    /// consumers with a known row range (`capture-pane -S/-E`) pay for that
    /// range alone. `cols`, `viewport_rows`, and `scrollback_rows` still
    /// describe the whole grid; `rows[0]` is physical row `start`.
    fn grid_snapshot_range(&self, start: usize, count: usize) -> Result<Grid, Error> {
        let cols = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_COLS)?;
        let viewport_rows = self.get_u16(ffi::GHOSTTY_TERMINAL_DATA_ROWS)?;
        let total_rows = self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_TOTAL_ROWS)?;
        let scrollback_rows = self.get_usize(ffi::GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS)?;
        let start = start.min(total_rows);
        let end = start.saturating_add(count).min(total_rows);
        let mut rows = Vec::with_capacity(end - start);

        for y in start..end {
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
                    ffi::GHOSTTY_CELL_WIDE_NARROW => CellWidth::Narrow,
                    ffi::GHOSTTY_CELL_WIDE_WIDE => CellWidth::Wide,
                    ffi::GHOSTTY_CELL_WIDE_SPACER_TAIL => CellWidth::SpacerTail,
                    ffi::GHOSTTY_CELL_WIDE_SPACER_HEAD => CellWidth::SpacerHead,
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
                    ffi::GHOSTTY_CELL_SEMANTIC_OUTPUT => CellSemantic::Output,
                    ffi::GHOSTTY_CELL_SEMANTIC_INPUT => CellSemantic::Input,
                    ffi::GHOSTTY_CELL_SEMANTIC_PROMPT => CellSemantic::Prompt,
                    _ => return Err(Error(ffi::GHOSTTY_INVALID_VALUE)),
                };
                let hyperlink =
                    self.grid_ref_hyperlink(&grid_ref, ffi::ghostty_grid_ref_hyperlink_uri)?;
                let hyperlink_id =
                    self.grid_ref_hyperlink(&grid_ref, ffi::ghostty_grid_ref_hyperlink_id)?;
                cells.push(GridCell {
                    text,
                    width,
                    semantic,
                    hyperlink,
                    hyperlink_id,
                    // libghostty-vt does not keep a tab's origin: by the time a
                    // cell is readable the tab is the blanks it painted.
                    tab: false,
                });
            }
            // Ghostty's rows are physical cells with no tmux allocation
            // boundary behind them, so both extents are the whole row and the
            // line flags it does not keep read as unset. `capture-pane -N`,
            // `-T` and `-F` see the difference; the engine is what answers them
            // faithfully.
            let size = cells.len();
            rows.push(GridRow {
                cells,
                wrapped,
                used: size,
                size,
                flags: RowFlags::default(),
            });
        }

        Ok(Grid {
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
    fn dump_plain(&self) -> Result<String, Error> {
        self.dump_plain_with(false)
    }

    /// Render the active screen as plain text with soft-wrapped rows rejoined
    /// into their logical lines. A row the terminal wrapped only because it hit
    /// the right margin is emitted as one line here, rather than split at the
    /// (invisible) wrap point — what a reader wants when reading back output.
    fn dump_plain_unwrapped(&self) -> Result<String, Error> {
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

    /// Render a contiguous range of full-screen rows as trimmed plain text
    /// without formatting rows outside that range. `start` is zero-based from
    /// the oldest history row. Single-row readers (`#{cursor_character}`)
    /// use this to avoid paying for the whole scrollback.
    fn dump_plain_rows(&self, start: usize, rows: usize, cols: u16) -> Result<String, Error> {
        if rows == 0 || cols == 0 {
            return Ok(String::new());
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
        let mut options = plain_formatter_options(false);
        options.selection = &selection as *const ffi::GhosttySelection as *const _;

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
    fn dump_vt(&self) -> Result<Vec<u8>, Error> {
        self.dump_vt_selection(None)
    }

    /// Render a contiguous range of full-screen rows as VT without formatting
    /// rows outside that range. `start` is zero-based from oldest history.
    fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> Result<Vec<u8>, Error> {
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

/// libghostty-vt's `GhosttyMouseAction` numbering. The seam's enum is hmux's,
/// so the ABI values are mapped here rather than baked into it.
fn ffi_action(action: MouseAction) -> i32 {
    match action {
        MouseAction::Press => 0,
        MouseAction::Release => 1,
        MouseAction::Motion => 2,
    }
}

/// libghostty-vt's `GhosttyMouseButton` numbering.
fn ffi_button(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 1,
        MouseButton::Right => 2,
        MouseButton::Middle => 3,
        MouseButton::WheelUp => 4,
        MouseButton::WheelDown => 5,
        MouseButton::Six => 6,
        MouseButton::Seven => 7,
        MouseButton::Eight => 8,
        MouseButton::Nine => 9,
        MouseButton::Ten => 10,
        MouseButton::Eleven => 11,
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

impl Drop for GhosttyScreen {
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

impl VtScreen for GhosttyScreen {
    /// libghostty-vt has its own parser, so the token goes back to the bytes it
    /// was built from. They are the original bytes, not a re-serialization, so
    /// nothing is lost in the round trip.
    fn apply(&mut self, token: &Token) {
        GhosttyScreen::write(self, &token.raw);
    }

    fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        Ok(GhosttyScreen::resize(self, cols, rows)?)
    }

    /// libghostty-vt keeps the modes but publishes no word of them, and a
    /// backend that cannot report state owes the reconstruction itself — a
    /// shadow tokenizer inside this module. Nothing asks: the default build
    /// runs the engine, and the differential harness compares grids and
    /// encoders. So this reports the modes a screen starts with, and the day
    /// something here needs better, that shadow is what it costs.
    fn modes(&self) -> u32 {
        super::screen::mode::CURSOR | super::screen::mode::WRAP
    }

    /// libghostty-vt decides for itself what clearing the screen does with the
    /// scrollback, so there is nothing here for `scroll-on-clear` to steer.
    fn set_options(&mut self, _options: ScreenOptions) {}

    /// libghostty-vt owns its scrollback and offers nothing equivalent, so this
    /// backend leaves the grid alone rather than approximating the move.
    fn trim_history_below_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn cursor_position(&self) -> io::Result<(u16, u16)> {
        Ok(GhosttyScreen::cursor_position(self)?)
    }

    fn cursor_visible(&self) -> io::Result<bool> {
        Ok(GhosttyScreen::cursor_visible(self)?)
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        Ok(GhosttyScreen::scrollback_rows(self)?)
    }

    fn grid_dims(&self) -> io::Result<GridDims> {
        Ok(GhosttyScreen::grid_dims(self)?)
    }

    fn grid_snapshot(&self) -> io::Result<Grid> {
        Ok(GhosttyScreen::grid_snapshot(self)?)
    }

    /// libghostty-vt exposes only the screen in use. The grid the alternate
    /// screen displaced is inside the library with no way to read it, so `-a`
    /// gets the same answer as a pane that never switched: there is none.
    fn inactive_snapshot(&self) -> io::Result<Option<(Grid, Vec<u8>)>> {
        Ok(None)
    }

    fn grid_snapshot_range(&self, start: usize, count: usize) -> io::Result<Grid> {
        Ok(GhosttyScreen::grid_snapshot_range(self, start, count)?)
    }

    fn dump_plain(&self) -> io::Result<String> {
        Ok(GhosttyScreen::dump_plain(self)?)
    }

    fn dump_plain_unwrapped(&self) -> io::Result<String> {
        Ok(GhosttyScreen::dump_plain_unwrapped(self)?)
    }

    fn dump_plain_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<String> {
        Ok(GhosttyScreen::dump_plain_rows(self, start, rows, cols)?)
    }

    fn dump_vt(&self) -> io::Result<Vec<u8>> {
        Ok(GhosttyScreen::dump_vt(self)?)
    }

    fn dump_vt_rows(&self, start: usize, rows: usize, cols: u16) -> io::Result<Vec<u8>> {
        Ok(GhosttyScreen::dump_vt_rows(self, start, rows, cols)?)
    }

    fn dump_vt_capture_rows(
        &self,
        start: usize,
        rows: usize,
        cols: u16,
        _extent: CaptureExtent,
    ) -> io::Result<Vec<u8>> {
        // libghostty-vt's rows have neither of tmux's two extents behind them,
        // so there is nothing for the requested one to select and the capture
        // and the redraw are the same bytes here. That difference is this
        // backend's to carry, not the server's.
        Ok(GhosttyScreen::dump_vt_rows(self, start, rows, cols)?)
    }
}

impl InputEncoder for GhosttyScreen {
    fn encode_key(&self, key: KeyEvent<'_>) -> io::Result<Vec<u8>> {
        Ok(GhosttyScreen::encode_key(self, key)?)
    }

    fn encode_mouse(&self, mouse: MouseEvent) -> io::Result<Vec<u8>> {
        Ok(GhosttyScreen::encode_mouse(self, mouse)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vt::input::{Key, MouseAction, MouseButton};

    #[test]
    fn key_encoder_tracks_terminal_cursor_mode() {
        let mut screen = GhosttyScreen::new(20, 5).expect("new screen");
        let up = KeyEvent {
            key: Key::ARROW_UP,
            shift: false,
            control: false,
            alt: false,
            text: None,
            unshifted_codepoint: None,
        };
        assert_eq!(screen.encode_key(up).unwrap(), b"\x1b[A");
        screen.write(b"\x1b[?1h");
        assert_eq!(screen.encode_key(up).unwrap(), b"\x1bOA");
    }

    #[test]
    fn mouse_encoder_tracks_terminal_mode_and_sgr_format() {
        let mut screen = GhosttyScreen::new(20, 5).expect("new screen");
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
        assert_eq!(
            screen.encode_mouse(press).expect("disabled encoding"),
            b"",
            "with no mouse mode set the program is told nothing"
        );

        screen.write(b"\x1b[?1000h\x1b[?1006h");
        assert_eq!(
            screen.encode_mouse(press).expect("SGR press"),
            b"\x1b[<0;3;4M"
        );
        assert_eq!(
            screen
                .encode_mouse(MouseEvent {
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
        let mut screen = GhosttyScreen::new(20, 5).expect("new screen");
        screen.write(b"\x1b[?1002h\x1b[?1006h");
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
            screen.encode_mouse(event).expect("modified wheel"),
            b"\x1b[<84;1;1M"
        );
    }

    #[test]
    fn plain_text_round_trips_through_the_grid() {
        let mut screen = GhosttyScreen::new(20, 5).expect("new screen");
        screen.write(b"hello world");
        let dump = screen.dump_plain().expect("dump");
        assert!(
            dump.contains("hello world"),
            "grid should show written text, got {dump:?}"
        );
    }

    #[test]
    fn tab_spaces_and_cursor_gap_render_as_the_same_blanks() {
        for input in [b"\tX".as_slice(), b"        X", b"\x1b[9GX"] {
            let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
            screen.write(input);
            assert_eq!(
                screen.dump_plain().expect("dump"),
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
            let mut screen = GhosttyScreen::new(12, 2).expect("new screen");
            screen.write(input);
            let snapshot = screen.grid_snapshot().expect("snapshot");
            assert_eq!(snapshot.cols, 12);
            assert_eq!(snapshot.viewport_rows, 2);
            assert_eq!(snapshot.rows.len(), 2);
            for cell in &snapshot.rows[0].cells[..8] {
                assert_eq!(cell.text == " ", written_spaces);
            }
            assert_eq!(snapshot.rows[0].cells[8].text, "X");
        }
    }

    /// `capture-pane -e` and the copy-mode/mouse hyperlink formats read the
    /// link off the cell, so the snapshot must carry both the URI and the id
    /// the program set explicitly. Implicit ids are internal and stay unset.
    #[test]
    fn grid_snapshot_carries_hyperlink_uri_and_explicit_id() {
        let mut screen = GhosttyScreen::new(12, 2).expect("new screen");
        screen.write(b"\x1b]8;id=x1;https://example.test\x1b\\a\x1b]8;;\x1b\\");
        screen.write(b"\x1b]8;;https://plain.test\x1b\\b\x1b]8;;\x1b\\c");
        let snapshot = screen.grid_snapshot().expect("snapshot");
        let cells = &snapshot.rows[0].cells;
        assert_eq!(cells[0].hyperlink.as_deref(), Some("https://example.test"));
        assert_eq!(cells[0].hyperlink_id.as_deref(), Some("x1"));
        assert_eq!(cells[1].hyperlink.as_deref(), Some("https://plain.test"));
        assert_eq!(cells[1].hyperlink_id, None, "implicit ids stay unreported");
        assert_eq!(cells[2].hyperlink, None);
    }

    #[test]
    fn grid_snapshot_retains_graphemes_width_and_soft_wraps() {
        let mut screen = GhosttyScreen::new(4, 3).expect("new screen");
        screen.write("e\u{301}界xy".as_bytes());
        let snapshot = screen.grid_snapshot().expect("snapshot");
        assert_eq!(snapshot.rows[0].cells[0].text, "e\u{301}");
        assert_eq!(snapshot.rows[0].cells[0].width, CellWidth::Narrow);
        assert_eq!(snapshot.rows[0].cells[1].text, "界");
        assert_eq!(snapshot.rows[0].cells[1].width, CellWidth::Wide);
        assert_eq!(snapshot.rows[0].cells[2].width, CellWidth::SpacerTail);
        assert_eq!(snapshot.rows[0].cells[3].text, "x");
        assert!(snapshot.rows[0].wrapped);
        assert_eq!(snapshot.rows[1].cells[0].text, "y");
    }

    #[test]
    fn csi_cursor_move_and_overwrite() {
        let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
        // Write "AAAAA", carriage-return to col 0, overwrite first two with "BB".
        screen.write(b"AAAAA\rBB");
        let dump = screen.dump_plain().expect("dump");
        assert!(dump.contains("BBAAA"), "CR + overwrite, got {dump:?}");
    }

    #[test]
    fn cursor_position_reports_inner_grid_coordinates() {
        let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
        screen.write(b"abc\r\nxy");
        assert_eq!(screen.cursor_position().expect("cursor position"), (2, 1));
    }

    #[test]
    fn newlines_produce_multiple_rows() {
        let mut screen = GhosttyScreen::new(20, 4).expect("new screen");
        screen.write(b"line1\r\nline2\r\nline3");
        let dump = screen.dump_plain().expect("dump");
        for expected in ["line1", "line2", "line3"] {
            assert!(dump.contains(expected), "missing {expected:?} in {dump:?}");
        }
    }

    #[test]
    fn sgr_color_codes_are_absorbed_not_shown() {
        let mut screen = GhosttyScreen::new(20, 2).expect("new screen");
        // Red "X" then reset — plain format must not leak the escape bytes.
        screen.write(b"\x1b[31mX\x1b[0m");
        let dump = screen.dump_plain().expect("dump");
        assert!(dump.contains('X'), "text present, got {dump:?}");
        assert!(
            !dump.contains('\x1b'),
            "no raw escapes in plain dump: {dump:?}"
        );
    }

    #[test]
    fn vt_dump_contains_text_and_escapes() {
        let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
        screen.write(b"hello");
        let vt = screen.dump_vt().expect("vt dump");
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
    fn vt_dump_carries_osc8_hyperlinks() {
        let mut screen = GhosttyScreen::new(40, 3).expect("new screen");
        screen.write(b"\x1b]8;;https://example.test\x1b\\link\x1b]8;;\x1b\\ after");
        let vt = screen.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        assert!(
            s.contains("\x1b]8;;https://example.test\x1b\\link\x1b]8;;\x1b\\"),
            "vt dump must re-open and close the OSC 8 hyperlink around the \
             linked cells, got {s:?}"
        );
    }

    /// The compositor splits a dump on newlines and repaints each row at its
    /// own screen position, so a row that ends inside a link must close it:
    /// an open OSC 8 would otherwise carry into whatever is drawn next,
    /// including another pane's row.
    #[test]
    fn vt_dump_closes_osc8_hyperlinks_at_every_row_end() {
        let mut screen = GhosttyScreen::new(40, 3).expect("new screen");
        screen.write(b"\x1b]8;;https://example.test\x1b\\ab\r\ncd\x1b]8;;\x1b\\");
        let vt = screen.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        for row in s.split("\r\n") {
            let opens = row.matches("\x1b]8;;https://example.test\x1b\\").count();
            let closes = row.matches("\x1b]8;;\x1b\\").count();
            assert_eq!(
                opens, closes,
                "every row must close the links it opens, got {row:?} in {s:?}"
            );
        }
    }

    #[test]
    fn vt_dump_separates_adjacent_osc8_hyperlinks() {
        let mut screen = GhosttyScreen::new(40, 3).expect("new screen");
        screen.write(
            b"\x1b]8;;https://one.test\x1b\\aa\x1b]8;;https://two.test\x1b\\bb\x1b]8;;\x1b\\",
        );
        let vt = screen.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        assert!(
            s.contains(
                "\x1b]8;;https://one.test\x1b\\aa\x1b]8;;\x1b\\\
                 \x1b]8;;https://two.test\x1b\\bb\x1b]8;;\x1b\\"
            ),
            "adjacent links must be closed and reopened between the runs, \
             got {s:?}"
        );
    }

    #[test]
    fn vt_dump_carries_osc8_explicit_id() {
        let mut screen = GhosttyScreen::new(40, 3).expect("new screen");
        screen.write(b"\x1b]8;id=x1;https://example.test\x1b\\link\x1b]8;;\x1b\\");
        let vt = screen.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        assert!(
            s.contains("\x1b]8;id=x1;https://example.test\x1b\\link"),
            "vt dump must preserve the explicit OSC 8 id, got {s:?}"
        );
    }

    #[test]
    fn cursor_visible_tracks_dectcem() {
        let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
        // Default: the cursor is visible.
        assert!(
            screen.cursor_visible().expect("query"),
            "default is visible"
        );
        // A TUI hides the cursor with DECTCEM reset (CSI ? 25 l)…
        screen.write(b"\x1b[?25l");
        assert!(
            !screen.cursor_visible().expect("query"),
            "hidden after ?25l"
        );
        // …and shows it again with DECTCEM set (CSI ? 25 h).
        screen.write(b"\x1b[?25h");
        assert!(
            screen.cursor_visible().expect("query"),
            "visible after ?25h"
        );
    }

    #[test]
    fn vt_dump_omits_cursor_visibility_mode() {
        // Guard the premise of the compositor fix: dump_vt carries the cursor
        // *position* but not its *visibility*, so the compositor must query
        // `cursor_visible()` and emit DECTCEM itself. If ghostty ever starts
        // emitting ?25 here, revisit the double-cursor handling in the compositor.
        let mut screen = GhosttyScreen::new(20, 3).expect("new screen");
        screen.write(b"\x1b[?25l");
        screen.write(b"hi");
        let vt = screen.dump_vt().expect("vt dump");
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
        let mut screen = GhosttyScreen::new(10, 4).expect("new screen");
        screen.write(b"abcdefghijKLMNOP");
        // The plain dump preserves the wrap as two rows…
        let wrapped = screen.dump_plain().expect("dump");
        assert_eq!(wrapped.lines().count(), 2, "wrapped: {wrapped:?}");
        // …while the unwrapped dump rejoins them into one logical line.
        let unwrapped = screen.dump_plain_unwrapped().expect("dump");
        assert_eq!(
            unwrapped.trim_end(),
            "abcdefghijKLMNOP",
            "unwrapped should be one line, got {unwrapped:?}"
        );
    }

    #[test]
    fn scrollback_rows_counts_history_beyond_the_viewport() {
        // A 4-row screen. Nothing scrolled yet → no scrollback.
        let mut screen = GhosttyScreen::new(80, 4).expect("new screen");
        screen.write(b"A\r\nB");
        assert_eq!(screen.scrollback_rows().expect("count"), 0);

        // Write 8 lines into the 4-row screen: 4 rows scroll into history.
        let mut screen = GhosttyScreen::new(80, 4).expect("new screen");
        screen.write(b"A\r\nB\r\nC\r\nD\r\nE\r\nF\r\nG\r\nH");
        assert_eq!(
            screen.scrollback_rows().expect("count"),
            4,
            "four rows (A–D) scrolled above the 4-row viewport"
        );

        // The dump carries history *plus* the viewport, oldest first: the count
        // is exactly how many leading rows a viewport-only consumer must skip.
        let vt = screen.dump_vt().expect("vt");
        let text = String::from_utf8_lossy(&vt);
        let rows: Vec<&str> = text.split('\n').collect();
        assert!(
            rows[0].starts_with('A'),
            "dump starts at oldest history row"
        );
        assert!(
            rows[screen.scrollback_rows().unwrap()].starts_with('E'),
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

        let mut screen = GhosttyScreen::new(80, VIEWPORT_ROWS as u16).expect("new screen");
        for n in 0..(REQUIRED_HISTORY_ROWS + VIEWPORT_ROWS) {
            screen.write(format!("SCROLLLINE_{n:04}\r\n").as_bytes());
        }

        let retained = screen.scrollback_rows().expect("scrollback row count");
        assert!(
            retained >= REQUIRED_HISTORY_ROWS,
            "native retained {retained} representative rows; expected at least \
             {REQUIRED_HISTORY_ROWS} for ordinary 80-column shell output"
        );
        let dump = screen.dump_plain().expect("history dump");
        assert!(
            dump.contains("SCROLLLINE_0000"),
            "the oldest row in the representative 4,000-line workload was discarded"
        );
    }

    #[test]
    fn vt_dump_preserves_colors() {
        let mut screen = GhosttyScreen::new(20, 2).expect("new screen");
        screen.write(b"\x1b[31mRED\x1b[0m");
        let vt = screen.dump_vt().expect("vt dump");
        let s = String::from_utf8_lossy(&vt);
        // VT should preserve the color via SGR, unlike plain.
        assert!(s.contains("RED"), "vt should contain text, got {s:?}");
        assert!(
            s.contains("\x1b["),
            "vt should contain SGR/CUP escapes, got {s:?}"
        );
    }
}

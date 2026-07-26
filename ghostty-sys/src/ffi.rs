//! Hand-written FFI declarations for the subset of the libghostty-vt C API we
//! use (terminal lifecycle + VT stream, formatters, and Unicode width).
//!
//! Mirrors `include/ghostty/vt/{types,terminal,formatter,mouse}.h` from the
//! vendored upstream `ghostty` main commit `2de5e7d3` (the 1.3.2-dev line).
//! Written by hand from the MIT-licensed headers rather than generated, so we
//! don't depend on `bindgen`/libclang. The `#[repr(C)]` layouts match the C
//! structs field-for-field; the sized structs (`size` first) are versioned by
//! the library via that `size` field.
//!
//! The terminal subset declared here (lifecycle + VT stream + grid/cell/row
//! inspection + the plain/VT formatter) remains byte-for-byte compatible with
//! the prior pin: its structs and function signatures are unchanged, and the
//! terminal header changes only affect options and data values unused here.
//! Unicode declarations mirror `include/ghostty/vt/unicode.h` from the vendored
//! source.

#![allow(non_camel_case_types)]

use std::os::raw::c_void;

// ---- result codes (types.h `GhosttyResult`, backed by c_int) ----------
pub const GHOSTTY_SUCCESS: i32 = 0;
pub const GHOSTTY_INVALID_VALUE: i32 = -2;
#[allow(dead_code)]
pub const GHOSTTY_OUT_OF_SPACE: i32 = -3;

// ---- content output format (types.h `GhosttyFormatterFormat`) ----------
pub const GHOSTTY_FORMATTER_FORMAT_PLAIN: i32 = 0;
pub const GHOSTTY_FORMATTER_FORMAT_VT: i32 = 1;

// ---- terminal data (terminal.h `GhosttyTerminalData`) -----------------
pub const GHOSTTY_TERMINAL_DATA_COLS: i32 = 1;
pub const GHOSTTY_TERMINAL_DATA_ROWS: i32 = 2;
pub const GHOSTTY_TERMINAL_DATA_CURSOR_X: i32 = 3;
pub const GHOSTTY_TERMINAL_DATA_CURSOR_Y: i32 = 4;
/// Whether the cursor is visible (DEC private mode 25, DECTCEM). Output: `bool*`.
pub const GHOSTTY_TERMINAL_DATA_CURSOR_VISIBLE: i32 = 7;
/// Whether any application mouse tracking mode is enabled. Output: `bool*`.
pub const GHOSTTY_TERMINAL_DATA_MOUSE_TRACKING: i32 = 11;
/// Terminal title set via OSC 0/2. Output: `GhosttyString*` (borrowed until the
/// next `ghostty_terminal_vt_write`/`_reset`; empty when no title has been set).
pub const GHOSTTY_TERMINAL_DATA_TITLE: i32 = 12;
/// Total rows in the active screen, including scrollback. Output: `size_t*`.
pub const GHOSTTY_TERMINAL_DATA_TOTAL_ROWS: i32 = 14;
/// Number of scrollback rows (total rows minus the viewport). Output: `size_t*`.
pub const GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS: i32 = 15;

// ---- screen.h cell/row data -------------------------------------------
pub type GhosttyCell = u64;
pub type GhosttyRow = u64;

pub const GHOSTTY_CELL_DATA_WIDE: i32 = 3;
pub const GHOSTTY_CELL_DATA_SEMANTIC_CONTENT: i32 = 9;
pub const GHOSTTY_CELL_WIDE_NARROW: i32 = 0;
pub const GHOSTTY_CELL_WIDE_WIDE: i32 = 1;
pub const GHOSTTY_CELL_WIDE_SPACER_TAIL: i32 = 2;
pub const GHOSTTY_CELL_WIDE_SPACER_HEAD: i32 = 3;
pub const GHOSTTY_CELL_SEMANTIC_OUTPUT: i32 = 0;
pub const GHOSTTY_CELL_SEMANTIC_INPUT: i32 = 1;
pub const GHOSTTY_CELL_SEMANTIC_PROMPT: i32 = 2;

pub const GHOSTTY_ROW_DATA_WRAP: i32 = 1;

// ---- types.h `GhosttyString` -------------------------------------------
/// A borrowed byte string (pointer + length); the memory is owned by the
/// library and only valid for the lifetime documented by the producing API.
#[repr(C)]
pub struct GhosttyString {
    pub ptr: *const u8,
    pub len: usize,
}

// ---- opaque handles ----------------------------------------------------
#[repr(C)]
pub struct GhosttyTerminalImpl {
    _private: [u8; 0],
}
pub type GhosttyTerminal = *mut GhosttyTerminalImpl;

#[repr(C)]
pub struct GhosttyKeyEncoderImpl {
    _private: [u8; 0],
}
pub type GhosttyKeyEncoder = *mut GhosttyKeyEncoderImpl;

#[repr(C)]
pub struct GhosttyKeyEventImpl {
    _private: [u8; 0],
}
pub type GhosttyKeyEvent = *mut GhosttyKeyEventImpl;

#[repr(C)]
pub struct GhosttyMouseEncoderImpl {
    _private: [u8; 0],
}
pub type GhosttyMouseEncoder = *mut GhosttyMouseEncoderImpl;

#[repr(C)]
pub struct GhosttyMouseEventImpl {
    _private: [u8; 0],
}
pub type GhosttyMouseEvent = *mut GhosttyMouseEventImpl;

#[repr(C)]
pub struct GhosttyFormatterImpl {
    _private: [u8; 0],
}
pub type GhosttyFormatter = *mut GhosttyFormatterImpl;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    pub x: u16,
    pub y: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GhosttyPointValue {
    pub coordinate: GhosttyPointCoordinate,
    pub padding: [u64; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPoint {
    pub tag: i32,
    pub value: GhosttyPointValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyGridRef {
    pub size: usize,
    pub node: *mut c_void,
    pub x: u16,
    pub y: u16,
}

#[repr(C)]
pub struct GhosttySelection {
    pub size: usize,
    pub start: GhosttyGridRef,
    pub end: GhosttyGridRef,
    pub rectangle: bool,
}

pub const GHOSTTY_POINT_TAG_SCREEN: i32 = 2;

// ---- terminal.h `GhosttyTerminalOptions` -------------------------------
#[repr(C)]
pub struct GhosttyTerminalOptions {
    /// Terminal width in cells. Must be > 0.
    pub cols: u16,
    /// Terminal height in cells. Must be > 0.
    pub rows: u16,
    /// Maximum scrollback lines to retain.
    pub max_scrollback: usize,
}

#[repr(C)]
pub struct GhosttyMousePosition {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct GhosttyMouseEncoderSize {
    pub size: usize,
    pub screen_width: u32,
    pub screen_height: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub padding_top: u32,
    pub padding_bottom: u32,
    pub padding_right: u32,
    pub padding_left: u32,
}

pub const GHOSTTY_MOUSE_ENCODER_OPT_SIZE: i32 = 2;
pub const GHOSTTY_MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED: i32 = 3;

// ---- formatter.h sized option structs ----------------------------------
#[repr(C)]
pub struct GhosttyFormatterScreenExtra {
    pub size: usize,
    pub cursor: bool,
    pub style: bool,
    pub hyperlink: bool,
    pub protection: bool,
    pub kitty_keyboard: bool,
    pub charsets: bool,
}

#[repr(C)]
pub struct GhosttyFormatterTerminalExtra {
    pub size: usize,
    pub palette: bool,
    pub modes: bool,
    pub scrolling_region: bool,
    pub tabstops: bool,
    pub pwd: bool,
    pub keyboard: bool,
    pub screen: GhosttyFormatterScreenExtra,
}

#[repr(C)]
pub struct GhosttyFormatterTerminalOptions {
    pub size: usize,
    pub emit: i32, // GhosttyFormatterFormat
    pub unwrap: bool,
    pub trim: bool,
    pub extra: GhosttyFormatterTerminalExtra,
    /// `const GhosttySelection*`; NULL formats the whole screen.
    pub selection: *const c_void,
}

extern "C" {
    pub fn ghostty_unicode_codepoint_width(codepoint: u32) -> u8;

    pub fn ghostty_unicode_grapheme_width(
        codepoints: *const u32,
        len: usize,
        width: *mut u8,
    ) -> usize;

    // Allocator arg is `const GhosttyAllocator*`; NULL selects the default.
    pub fn ghostty_terminal_new(
        allocator: *const c_void,
        terminal: *mut GhosttyTerminal,
        options: GhosttyTerminalOptions,
    ) -> i32;

    pub fn ghostty_terminal_free(terminal: GhosttyTerminal);

    pub fn ghostty_terminal_vt_write(terminal: GhosttyTerminal, data: *const u8, len: usize);

    pub fn ghostty_terminal_resize(
        terminal: GhosttyTerminal,
        cols: u16,
        rows: u16,
        cell_width_px: u32,
        cell_height_px: u32,
    ) -> i32;

    pub fn ghostty_terminal_get(terminal: GhosttyTerminal, data: i32, out: *mut c_void) -> i32;

    pub fn ghostty_key_encoder_new(
        allocator: *const c_void,
        encoder: *mut GhosttyKeyEncoder,
    ) -> i32;
    pub fn ghostty_key_encoder_free(encoder: GhosttyKeyEncoder);
    pub fn ghostty_key_encoder_setopt_from_terminal(
        encoder: GhosttyKeyEncoder,
        terminal: GhosttyTerminal,
    );
    pub fn ghostty_key_encoder_encode(
        encoder: GhosttyKeyEncoder,
        event: GhosttyKeyEvent,
        out_buf: *mut i8,
        out_buf_size: usize,
        out_len: *mut usize,
    ) -> i32;

    pub fn ghostty_key_event_new(allocator: *const c_void, event: *mut GhosttyKeyEvent) -> i32;
    pub fn ghostty_key_event_free(event: GhosttyKeyEvent);
    pub fn ghostty_key_event_set_action(event: GhosttyKeyEvent, action: i32);
    pub fn ghostty_key_event_set_key(event: GhosttyKeyEvent, key: i32);
    pub fn ghostty_key_event_set_mods(event: GhosttyKeyEvent, mods: u16);
    pub fn ghostty_key_event_set_consumed_mods(event: GhosttyKeyEvent, mods: u16);
    pub fn ghostty_key_event_set_utf8(event: GhosttyKeyEvent, utf8: *const i8, len: usize);
    pub fn ghostty_key_event_set_unshifted_codepoint(event: GhosttyKeyEvent, codepoint: u32);

    pub fn ghostty_mouse_encoder_new(
        allocator: *const c_void,
        encoder: *mut GhosttyMouseEncoder,
    ) -> i32;
    pub fn ghostty_mouse_encoder_free(encoder: GhosttyMouseEncoder);
    pub fn ghostty_mouse_encoder_setopt(
        encoder: GhosttyMouseEncoder,
        option: i32,
        value: *const c_void,
    );
    pub fn ghostty_mouse_encoder_setopt_from_terminal(
        encoder: GhosttyMouseEncoder,
        terminal: GhosttyTerminal,
    );
    pub fn ghostty_mouse_encoder_encode(
        encoder: GhosttyMouseEncoder,
        event: GhosttyMouseEvent,
        out_buf: *mut i8,
        out_buf_size: usize,
        out_len: *mut usize,
    ) -> i32;

    pub fn ghostty_mouse_event_new(allocator: *const c_void, event: *mut GhosttyMouseEvent) -> i32;
    pub fn ghostty_mouse_event_free(event: GhosttyMouseEvent);
    pub fn ghostty_mouse_event_set_action(event: GhosttyMouseEvent, action: i32);
    pub fn ghostty_mouse_event_set_button(event: GhosttyMouseEvent, button: i32);
    pub fn ghostty_mouse_event_clear_button(event: GhosttyMouseEvent);
    pub fn ghostty_mouse_event_set_mods(event: GhosttyMouseEvent, mods: u16);
    pub fn ghostty_mouse_event_set_position(
        event: GhosttyMouseEvent,
        position: GhosttyMousePosition,
    );

    pub fn ghostty_terminal_grid_ref(
        terminal: GhosttyTerminal,
        point: GhosttyPoint,
        out_ref: *mut GhosttyGridRef,
    ) -> i32;

    pub fn ghostty_grid_ref_cell(
        grid_ref: *const GhosttyGridRef,
        out_cell: *mut GhosttyCell,
    ) -> i32;

    pub fn ghostty_grid_ref_row(grid_ref: *const GhosttyGridRef, out_row: *mut GhosttyRow) -> i32;

    pub fn ghostty_grid_ref_graphemes(
        grid_ref: *const GhosttyGridRef,
        buf: *mut u32,
        buf_len: usize,
        out_len: *mut usize,
    ) -> i32;
    pub fn ghostty_grid_ref_hyperlink_uri(
        grid_ref: *const GhosttyGridRef,
        out_buf: *mut u8,
        buf_len: usize,
        out_len: *mut usize,
    ) -> i32;
    pub fn ghostty_grid_ref_hyperlink_id(
        grid_ref: *const GhosttyGridRef,
        out_buf: *mut u8,
        buf_len: usize,
        out_len: *mut usize,
    ) -> i32;

    pub fn ghostty_cell_get(cell: GhosttyCell, data: i32, out: *mut c_void) -> i32;

    pub fn ghostty_row_get(row: GhosttyRow, data: i32, out: *mut c_void) -> i32;

    pub fn ghostty_formatter_terminal_new(
        allocator: *const c_void,
        formatter: *mut GhosttyFormatter,
        terminal: GhosttyTerminal,
        options: GhosttyFormatterTerminalOptions,
    ) -> i32;

    pub fn ghostty_formatter_format_buf(
        formatter: GhosttyFormatter,
        buf: *mut u8,
        buf_len: usize,
        out_written: *mut usize,
    ) -> i32;

    pub fn ghostty_formatter_free(formatter: GhosttyFormatter);
}

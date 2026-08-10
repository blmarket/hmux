//! hmux's terminal-emulation layer.
//!
//! A pane's byte stream is parsed exactly once, by [`Parser`] — the DEC ANSI
//! state machine tmux's `input.c` implements. [`Observer`] turns those tokens
//! into the two things the server needs: the bytes the screen should apply,
//! and the ordered events everything else reacts to (query replies, OSC state,
//! mode changes, bells, clipboard, passthrough, titles).
//!
//! The grid those bytes land on is [`PaneScreen`], hmux's own port of tmux's
//! grid and screen model. It is the implementation, not one of several: there
//! is no backend to select, so its operations are inherent methods and the
//! ones that cannot fail say so in their signatures.
//!
//! # The contract
//!
//! The interface is the re-export list below and nothing else: every public
//! item is named in this file, so the whole surface is managed in one place.
//! The groups partition what the daemon needs from terminal emulation by
//! consumer context, one vocabulary each: the tokenizer for anything that
//! needs sequence framing without pane semantics, the observer for the pane
//! path, the screen for the grid and the values reading it produces, the key
//! and mouse identities an encode takes, and the width policy for measuring.
//! The groups stay separate because they change for different reasons —
//! owning the width tables should not mean owning the grid.
//!
//! Outbound there is nothing: this crate depends on no server code and on no
//! external crate at all. Every type crossing the boundary is owned here —
//! [`OutputPolicy`], [`ScreenOptions`], the event and token types — and the
//! server pushes resolved option values in rather than the emulator reading
//! server state.
//!
//! One caveat: the width policy is process-global mutable state, mirroring
//! tmux's global options — there is one width policy per process, not one per
//! screen.

mod engine;
mod input;
mod observer;
mod parser;
mod screen;
mod sixel;
mod vis;
mod width;
mod x11_colour;

/// The tmux release whose behavior hmux implements — product identity shared
/// by the emulator's XTVERSION reply and, via the daemon's re-export, its
/// command language (`#{version}`). Conformance is pinned to this version, so
/// an application that special-cases a terminal by version has to see the
/// same answer the command language claims to implement.
pub const TMUX_VERSION: &str = "3.7b";

// The tokenizer: bytes to framed tokens, lossless and semantics-free.
pub use parser::{tokenize, Param, Parser, StringEnd, Token, TokenKind};

// The pane path: tokens to screen tokens plus ordered events, options applied.
pub use observer::{
    base64_decode_strict, decrqss_reply, parse_packed_colour, ClipboardEvent, CursorShape, Event,
    Observed, Observer, OscUpdate, OutputPolicy, PassthroughPolicy, BACKGROUND_COLOR_QUERY,
};

// The screen: the grid itself and the values reading it produces.
pub use screen::{
    mode, CaptureExtent, CellSemantic, CellWidth, Grid, GridCell, GridDims, GridRow, PaneScreen,
    RowFlags, ScreenImage, ScreenOptions,
};

// The key and mouse identities an encode takes.
pub use input::{encode_key_default_modes, Key, KeyEvent, MouseAction, MouseButton, MouseEvent};

// The width policy: how many cells a codepoint occupies.
pub use width::{codepoint_width, set_codepoint_widths, set_variation_selector_always_wide};

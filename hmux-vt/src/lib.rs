//! hmux's terminal-emulation layer.
//!
//! The emulator is [`Terminal`]: one chunk of a pane's byte stream in, the
//! ordered [`TerminalEvent`] list the server has to act on out. Inside it the
//! stream is parsed exactly once — by the DEC ANSI state machine tmux's
//! `input.c` implements — the pane's options are applied, and the surviving
//! output lands on the screen. Tokens, and the interleaving of applying them
//! with observing them, are internal.
//!
//! The grid itself is [`PaneScreen`], hmux's own port of tmux's grid and
//! screen model, reachable through the terminal for everything that reads it
//! back out. It is the implementation, not one of several: there is no
//! backend to select, so its operations are inherent methods and the ones
//! that cannot fail say so in their signatures.
//!
//! # The contract
//!
//! The interface is the re-export list below and nothing else: every public
//! item is named in this file, so the whole surface is managed in one place.
//! The groups partition what the daemon needs from terminal emulation by
//! consumer context, one vocabulary each: the terminal for the pane path, the
//! screen for the grid and the values reading it produces, the key and mouse
//! identities an encode takes, the answer scanner for the client tty path,
//! the model for out-of-process harnesses, and the width policy for
//! measuring. The groups stay separate because they change for different
//! reasons — owning the width tables should not mean owning the grid.
//!
//! Outbound there is nothing: this crate depends on no server code and on no
//! external crate at all. Every type crossing the boundary is owned here —
//! [`OutputPolicy`], [`ScreenOptions`], the event types — and the server
//! pushes resolved option values in rather than the emulator reading server
//! state. Events are self-contained for the same reason in reverse: screen
//! state an event reports is captured at the event's point in the stream, so
//! the server acts on values, not on a screen it has to read at the right
//! moment.
//!
//! One caveat: the width policy is process-global mutable state, mirroring
//! tmux's global options — there is one width policy per process, not one per
//! screen.

mod answer;
mod engine;
mod input;
mod model;
mod observer;
mod parser;
mod screen;
mod scroll;
mod sixel;
mod terminal;
mod vis;
mod width;
mod x11_colour;

/// The tmux release whose behavior hmux implements — product identity shared
/// by the emulator's XTVERSION reply and, via the daemon's re-export, its
/// command language (`#{version}`). Conformance is pinned to this version, so
/// an application that special-cases a terminal by version has to see the
/// same answer the command language claims to implement.
pub const TMUX_VERSION: &str = "3.7b";

// The pane path: one chunk of the pane's byte stream in, ordered
// self-contained events out, with the screen inside.
pub use terminal::{Terminal, TerminalEvent};

// The vocabulary the events carry, the option values pushed in, and the
// OSC-set pane state read back out.
pub use observer::{
    parse_packed_colour, ClipboardEvent, CursorShape, OscState, OutputPolicy, PassthroughPolicy,
    BACKGROUND_COLOR_QUERY,
};
pub use parser::StringEnd;

// The screen: the grid itself and the values reading it produces.
pub use screen::{
    mode, CaptureExtent, CellSemantic, CellWidth, Grid, GridCell, GridDims, GridRow, PaneScreen,
    RowFlags, ScreenImage, ScreenOptions,
};

// The key and mouse identities an encode takes.
pub use input::{encode_key_default_modes, Key, KeyEvent, MouseAction, MouseButton, MouseEvent};

// The client tty path: the outer terminal's answers, picked out of its input.
pub use answer::{AnswerScanner, TerminalAnswer};

// The model for out-of-process harnesses: feed bytes, read the screen back.
pub use model::TerminalModel;

// The width policy: how many cells a codepoint occupies.
pub use width::{codepoint_width, set_codepoint_widths, set_variation_selector_always_wide};

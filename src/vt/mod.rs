//! hmux's terminal-emulation layer.
//!
//! A pane's byte stream is parsed exactly once, by [`parser::Parser`] — the DEC
//! ANSI state machine tmux's `input.c` implements. [`observer::Observer`] turns
//! those tokens into the two things the server needs: the bytes the screen
//! should apply, and the ordered events everything else reacts to (query
//! replies, OSC state, mode changes, bells, clipboard, passthrough, titles).
//!
//! The screen itself sits behind [`screen::VtScreen`], mouse encoding behind
//! [`input::InputEncoder`], and character widths behind [`width`]. They are
//! two traits and a module rather than one, because they change for different
//! reasons: a grid rewrite should not drag input encoding with it, and owning
//! the width tables should not mean owning the grid.
//!
//! # The contract
//!
//! Inbound, the interface is the five `pub(crate)` submodules — [`parser`],
//! [`observer`], [`screen`], [`input`], [`width`] — plus [`PaneScreen`], the
//! implementation this build ships. Server code calls trait methods, so that
//! alias is the only place a backend is chosen; `engine` (hmux's own port of
//! tmux's grid and screen model) and `ghostty` (the libghostty-vt backend it
//! replaced, still buildable behind the `ghostty` feature for `differential`
//! to diff against) are private behind it.
//!
//! Outbound there is nothing: vt never depends on the server module. Every
//! type crossing the seam is vt-owned — [`observer::OutputPolicy`],
//! [`screen::ScreenOptions`], the event and token types — and the server
//! pushes resolved option values in rather than vt reading server state.
//!
//! One caveat: [`width`] is process-global mutable state, mirroring tmux's
//! global options — there is one width policy per process, not one per screen.
//!
//! `hmux::model::TerminalModel` is the only `pub` window onto this seam, and
//! it is not a compatibility contract.

#[cfg(all(test, feature = "ghostty"))]
mod differential;
mod engine;
#[cfg(feature = "ghostty")]
mod ghostty;
pub(crate) mod input;
pub(crate) mod observer;
pub(crate) mod parser;
pub(crate) mod screen;
mod vis;
pub(crate) mod width;
mod x11_colour;

/// The screen implementation this build ships.
pub(crate) type PaneScreen = engine::backend::EngineScreen;

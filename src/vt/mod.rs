//! hmux's terminal-emulation layer.
//!
//! A pane's byte stream is parsed exactly once, by [`parser::Parser`] — the DEC
//! ANSI state machine tmux's `input.c` implements. [`observer::Observer`] turns
//! those tokens into the two things the server needs: the bytes the screen
//! should apply, and the ordered events everything else reacts to (query
//! replies, OSC state, mode changes, bells, clipboard, passthrough, titles).
//!
//! The screen itself sits behind [`screen::VtScreen`], key and mouse encoding
//! behind [`input::InputEncoder`], and character widths behind [`width`]. They
//! are three traits and a module rather than one, because they change for
//! different reasons: a grid rewrite should not drag key encoding with it, and
//! owning the width tables should not mean owning the grid.
//!
//! [`PaneScreen`] names the implementation this build ships. Server code calls
//! trait methods, so that alias is the only place a backend is chosen.
//!
//! The backend is [`engine`], hmux's own port of tmux's grid and screen model.
//! The libghostty-vt backend it replaced is still buildable behind the
//! `ghostty` feature, which is what [`differential`] diffs against; nothing in
//! the default build links it, and the default build needs only the Rust
//! toolchain.

#[cfg(all(test, feature = "ghostty"))]
mod differential;
pub(crate) mod engine;
#[cfg(feature = "ghostty")]
pub(crate) mod ghostty;
pub(crate) mod input;
pub(crate) mod observer;
pub(crate) mod parser;
pub(crate) mod screen;
pub(crate) mod width;

/// The screen implementation this build ships.
pub(crate) type PaneScreen = engine::backend::EngineScreen;

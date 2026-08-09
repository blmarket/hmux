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
//! The interface is this crate's `pub` surface and nothing else: the six
//! modules [`parser`], [`observer`], [`screen`], [`input`], [`width`] and
//! [`sixel`], plus [`EngineScreen`] (aliased [`PaneScreen`], the implementation
//! this build ships) and [`TMUX_VERSION`]. Consumers call trait methods, so
//! that alias is the only place a backend is chosen; `engine` (hmux's own port
//! of tmux's grid and screen model) and `ghostty` (the libghostty-vt backend it
//! replaced, still buildable behind the `ghostty` feature for `differential`
//! to diff against) are private behind it.
//!
//! [`screen::VtScreen`] and [`input::InputEncoder`] are open public traits
//! under a semver promise; see their documentation.
//!
//! Outbound there is nothing: this crate depends on no server code and, in the
//! default build, on no external crate at all. Every type crossing the seam is
//! owned here — [`observer::OutputPolicy`], [`screen::ScreenOptions`], the
//! event and token types — and the server pushes resolved option values in
//! rather than the emulator reading server state.
//!
//! One caveat: [`width`] is process-global mutable state, mirroring tmux's
//! global options — there is one width policy per process, not one per screen.

#[cfg(all(test, feature = "ghostty"))]
mod differential;
mod engine;
#[cfg(feature = "ghostty")]
mod ghostty;
pub mod input;
pub mod observer;
pub mod parser;
pub mod screen;
pub mod sixel;
mod vis;
pub mod width;
mod x11_colour;

/// The tmux release whose behavior hmux implements — product identity shared
/// by the emulator's XTVERSION reply and, via the daemon's re-export, its
/// command language (`#{version}`). Conformance is pinned to this version, so
/// an application that special-cases a terminal by version has to see the
/// same answer the command language claims to implement.
pub const TMUX_VERSION: &str = "3.7b";

pub use engine::backend::EngineScreen;

/// The screen implementation this build ships.
pub type PaneScreen = EngineScreen;

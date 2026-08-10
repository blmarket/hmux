//! hmux's terminal-emulation layer.
//!
//! A pane's byte stream is parsed exactly once, by [`parser::Parser`] — the DEC
//! ANSI state machine tmux's `input.c` implements. [`observer::Observer`] turns
//! those tokens into the two things the server needs: the bytes the screen
//! should apply, and the ordered events everything else reacts to (query
//! replies, OSC state, mode changes, bells, clipboard, passthrough, titles).
//!
//! The grid those bytes land on is [`PaneScreen`], hmux's own port of tmux's
//! grid and screen model. It is the implementation, not one of several: there
//! is no backend to select, so its operations are inherent methods and the
//! ones that cannot fail say so in their signatures.
//!
//! # The contract
//!
//! The interface is this crate's `pub` surface and nothing else: the modules
//! [`parser`], [`observer`], [`screen`], [`input`] and [`width`], plus
//! [`PaneScreen`] and [`TMUX_VERSION`]. The modules stay separate because they
//! change for different reasons — owning the width tables should not mean
//! owning the grid — and [`screen`] holds the values a screen read produces
//! while [`input`] holds the key and mouse identities an encode takes.
//!
//! Outbound there is nothing: this crate depends on no server code and on no
//! external crate at all. Every type crossing the boundary is owned here —
//! [`observer::OutputPolicy`], [`screen::ScreenOptions`], the event and token
//! types — and the server pushes resolved option values in rather than the
//! emulator reading server state.
//!
//! One caveat: [`width`] is process-global mutable state, mirroring tmux's
//! global options — there is one width policy per process, not one per screen.

mod engine;
pub mod input;
pub mod observer;
pub mod parser;
pub mod screen;
mod sixel;
mod vis;
pub mod width;
mod x11_colour;

/// The tmux release whose behavior hmux implements — product identity shared
/// by the emulator's XTVERSION reply and, via the daemon's re-export, its
/// command language (`#{version}`). Conformance is pinned to this version, so
/// an application that special-cases a terminal by version has to see the
/// same answer the command language claims to implement.
pub const TMUX_VERSION: &str = "3.7b";

pub use engine::backend::PaneScreen;

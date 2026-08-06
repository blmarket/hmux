//! The in-house terminal engine.
//!
//! tmux's `grid.c` and `screen-write.c` are the reference, and the port is
//! deliberately literal: same cell layout, same two per-row lengths, same
//! scrolling rules. Where hmux's own semantics and tmux's cannot both hold, the
//! divergence is documented rather than smoothed over.
//!
//! This is not the shipped backend. It is built behind
//! [`super::screen::VtScreen`] alongside the libghostty-vt one, and flipping
//! the default is a separate, signed-off decision.

// The engine implements the seam but is not the backend this build ships, so
// nothing calls it outside its own tests. The allowance comes off when it
// becomes the default — the plan's Phase 3, which needs human signoff.
#![allow(dead_code)]

pub(crate) mod backend;
pub(crate) mod cell;
pub(crate) mod dispatch;
pub(crate) mod dump;
pub(crate) mod grid;
pub(crate) mod keys;
pub(crate) mod screen;

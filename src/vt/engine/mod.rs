//! The in-house terminal engine.
//!
//! tmux's `grid.c` and `screen-write.c` are the reference, and the port is
//! deliberately literal: same cell layout, same two per-row lengths, same
//! scrolling rules. Where hmux's own semantics and tmux's cannot both hold, the
//! divergence is documented rather than smoothed over.
//!
//! This is the shipped backend, reached through [`super::screen::VtScreen`]
//! and named by [`super::PaneScreen`].

pub(crate) mod backend;
pub(crate) mod cell;
mod combine;
pub(crate) mod dispatch;
pub(crate) mod dump;
pub(crate) mod grid;
pub(crate) mod hyperlinks;
pub(crate) mod keys;
pub(crate) mod screen;

//! The in-house terminal engine.
//!
//! tmux's `grid.c` and `screen-write.c` are the reference, and the port is
//! deliberately literal: same cell layout, same two per-row lengths, same
//! scrolling rules. Where hmux's own semantics and tmux's cannot both hold, the
//! divergence is documented rather than smoothed over.
//!
//! This is what a pane's grid actually is; [`crate::screen::PaneScreen`] is its
//! face.

pub mod backend;
pub mod cell;
mod combine;
pub mod dispatch;
pub mod dump;
pub mod grid;
pub mod hyperlinks;
pub mod images;
pub mod keys;
pub mod screen;

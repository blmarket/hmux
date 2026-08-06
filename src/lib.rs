//! hmux — a native tmux-compatible server.
//!
//! hmux binds its own unix socket and speaks tmux's imsg control protocol, so a
//! stock `tmux attach -S hmux.sock` can use native hmux sessions and panes.
//!
//! Modules:
//! - [`event_loop`] — readiness-driven event-loop engine.
//! - [`server`] — shared tmux command, state, and terminal engine.
//! - [`tmux`] — message layer, codec, server traits, and compatibility re-exports.
//! - [`observability`] — versioned pane observation contracts.
//! - [`integration`] — prototype consumers of optional runtime capabilities.
//! - [`model`] — a terminal model for out-of-process test harnesses.
//! - [`serve`] — listeners and connection lifecycle management.
//! - `vt` — the pane tokenizer, the terminal-emulation seam, and its backend.

pub mod error;
pub(crate) mod event_loop;
pub mod integration;
pub mod model;
pub mod observability;
#[allow(dead_code)]
mod platform;
pub mod serve;
pub(crate) mod server;
pub mod tmux;
pub(crate) mod vt;

pub use error::{Error, Result};

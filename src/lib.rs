//! hmux — a native tmux-compatible server.
//!
//! hmux binds its own unix socket and speaks tmux's imsg control protocol, so a
//! stock `tmux attach -S hmux.sock` can use native hmux sessions and panes.
//!
//! See `design.md` for the full design, including the important "Scope & limits"
//! section (interactive terminal I/O bypasses this layer via a passed tty fd).
//!
//! Modules:
//! - `common` — engine-agnostic event-loop building blocks.
//! - [`event_loop`] — readiness-driven event-loop engine.
//! - [`native`] — native libghostty-vt engine.
//! - [`tmux`] — message layer, codec, server traits, and compatibility re-exports.
//! - [`observability`] — versioned, native-runtime pane observation contracts.
//! - [`integration`] — prototype consumers of optional runtime capabilities.
//! - [`serve`] — the listener and per-connection pairing loop.

#[allow(dead_code)]
pub(crate) mod common;
pub mod error;
#[allow(dead_code)]
pub(crate) mod event_loop;
pub(crate) mod native;
/// Safe wrapper over libghostty-vt, the terminal-emulation core of the native
/// path. Lives in the standalone `ghostty-sys` crate (which owns the raw FFI and
/// the build/link logic); re-exported here so `hmux::ghostty::*` is unchanged.
pub use ghostty_sys as ghostty;
pub mod integration;
pub mod observability;
#[allow(dead_code)]
mod platform;
pub mod serve;
pub mod tmux;

pub use error::{Error, Result};

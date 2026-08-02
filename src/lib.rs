//! hmux — a native tmux-compatible server.
//!
//! hmux binds its own unix socket and speaks tmux's imsg control protocol, so a
//! stock `tmux attach -S hmux.sock` can use native hmux sessions and panes.
//!
//! Modules:
//! - [`event_loop`] — readiness-driven event-loop engine.
//! - [`server`] — shared tmux command, state, and terminal engine.
//! - [`native`] — blocking/threaded runtime adapter.
//! - [`tmux`] — message layer, codec, server traits, and compatibility re-exports.
//! - [`observability`] — versioned, native-runtime pane observation contracts.
//! - [`integration`] — prototype consumers of optional runtime capabilities.
//! - [`serve`] — listeners and connection lifecycle management.

pub mod error;
#[allow(dead_code)]
pub(crate) mod event_loop;
pub(crate) mod native;
pub(crate) mod server;
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

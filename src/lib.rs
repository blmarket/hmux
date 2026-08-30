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
//!
//! The pane tokenizer and the terminal emulation live in the `hmux-vt` crate;
//! the async runtime layer (`AsyncFd`, reactor, timers, tasks) lives in the
//! `hmux-rt` crate; the agent detectors, the observability contracts they read
//! through, and the platform process probing live in the `hmux-agent` crate,
//! re-exported here at their original paths so a second tmux-compatible server
//! can host the same integration. The daemon consumes all three only through
//! those crates' public surfaces. The dataflow primitives composed over those
//! leaves — completions, notifies, `select`/`join` — are the daemon's own, in
//! [`sync`].

// Leaves the daemon writes itself park non-`Send` wakers, the same contract
// as `hmux-rt`.
#![feature(local_waker)]

pub mod error;
pub(crate) mod event_loop;
pub mod serve;
pub(crate) mod server;
pub(crate) mod sync;
pub mod tmux;

pub use error::{Error, Result};

// The agent detectors, the pane observability contracts they read through, and
// the process probing both need live in `hmux-agent`, so a second
// tmux-compatible server can host the same integration. They keep their
// original paths here.
pub use hmux_agent::{integration, observability};
pub(crate) use hmux_agent::{pane_class, platform};

pub(crate) use hmux_vt::TMUX_VERSION;

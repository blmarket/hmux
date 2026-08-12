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
//! the daemon consumes them only through that crate's public surface.

// The loop's task set parks non-`Send` wakers on the event queue.
#![feature(local_waker)]

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

/// The event loop's `Future` executor.
///
/// Exposed for the demo in `examples/tasks.rs`: the daemon itself reaches these
/// through the loop, not through this module.
pub mod tasks {
    pub use crate::event_loop::reactor::{Interest, Readiness};
    pub use crate::event_loop::tasks::{
        join, sleep, sleep_until, AsyncFd, Join, ReadinessFuture, Sleep, TaskHandle, TaskRuntime,
    };
    pub use crate::server::task::{completion_pair, Completion, CompletionSender};
}

pub use error::{Error, Result};

pub(crate) use hmux_vt::TMUX_VERSION;

//! Single-threaded async runtime for the hmux daemon.
//!
//! This crate does not intend to support multi-threaded, thus no Send, no Sync.

#![feature(local_waker)]

mod handoff;
mod reactor;
mod runtime;
mod signals;
mod tasks;
mod timer;

pub use reactor::{Interest, Readiness};
pub use runtime::TaskRuntime;
pub use signals::Signals;
pub use tasks::{AsyncFd, JoinError, JoinHandle, TaskHandle, TaskId, sleep, sleep_until};

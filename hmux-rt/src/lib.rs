//! Single-threaded async runtime for the hmux daemon.
//!
//! [`TaskRuntime`] is the host: the daemon runs its whole event loop through
//! `dispatch`/`poll` turns, and every event source is a task holding a leaf
//! from this crate — an [`AsyncFd`] registration or a [`sleep`] deadline.
//! README.md records the scope and the boundary rules, including why the
//! `Waker` half of every poll context is inert: only `cx.local_waker()` is
//! wired, and **a leaf that parks `cx.waker()` never wakes**.

#![feature(local_waker)]

mod handoff;
mod reactor;
mod runtime;
mod tasks;
mod timer;

pub use reactor::{Interest, Readiness};
pub use runtime::TaskRuntime;
pub use tasks::{sleep, sleep_until, AsyncFd, JoinError, JoinHandle, TaskHandle, TaskId};

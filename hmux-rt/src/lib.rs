//! Single-threaded async runtime for the hmux daemon.
//!
//! [`TaskRuntime`] is the host: the daemon runs its whole event loop through
//! `dispatch`/`poll` turns, and every event source is a task holding a leaf
//! from this crate — an [`AsyncFd`] registration or a [`sleep`] deadline.
//! README.md records the scope and the boundary rules, including why the
//! `Waker` half of every poll context is inert: only `cx.local_waker()` is
//! wired, and **a leaf that parks `cx.waker()` never wakes**.
//!
//! Scope is runtime capability, nothing above it. The leaves here are the ones
//! that cannot be written from outside — they reach the reactor, the timer
//! queue or the wake path. Generic dataflow over them, `map` and `merge` and
//! the rest, needs none of that and is not this crate's to own; it belongs to
//! whoever is composing.
//!
//! The reactor, timer queue, and task-set plumbing behind the runtime are
//! implementation detail and stay private.

#![feature(local_waker)]

mod handoff;
mod reactor;
mod runtime;
mod task_loop;
mod tasks;
mod timer;

pub use reactor::{Interest, Readiness};
pub use runtime::TaskRuntime;
pub use tasks::{sleep, sleep_until, AsyncFd, JoinError, JoinHandle, TaskHandle, TaskId};

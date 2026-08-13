//! Single-threaded async runtime layer for a host-owned event loop.
//!
//! Futures, leaves, and wake plumbing that embed into an event loop the host
//! owns; the task set never runs a queue of its own behind the host's back.
//! README.md records the scope and the boundary rules, including why the
//! `Waker` half of every poll context is inert: only `cx.local_waker()` is
//! wired, and **a leaf that parks `cx.waker()` never wakes**.
//!
//! Scope is runtime capability, nothing above it. The leaves here are the ones
//! that cannot be written from outside — they reach the reactor, the timer
//! queue or the wake path. Generic dataflow over them, `map` and `merge` and
//! the rest, needs none of that and is not this crate's to own; it belongs to
//! whoever is composing. `examples/async_iter/` carries its own set.

#![feature(async_iterator, local_waker)]

mod completion;
mod notify;
mod reactor;
mod runtime;
mod task_loop;
mod tasks;
mod timer;

pub use completion::{completion_pair, Completion, CompletionSender, WakeFn};
pub use notify::{Notified, Notify};
pub use reactor::{Interest, MioReactor, PollResult, Reactor, Readiness, Ready, Token};
pub use runtime::TaskRuntime;
pub use task_loop::TaskLoop;
pub use tasks::{
    join, select, sleep, sleep_until, yield_now, AsyncFd, Either, Join, JoinError, JoinHandle,
    ReadinessFuture, ReadinessIter, Select, Sleep, TaskEvent, TaskHandle, TaskId, TaskSet,
    WakeSink, YieldNow,
};
pub use timer::{ExpiredTimer, TimerId, TimerQueue};

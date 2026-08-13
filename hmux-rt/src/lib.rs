//! Single-threaded async runtime layer for a host-owned event loop.
//!
//! Futures, leaves, and wake plumbing that embed into an event loop the host
//! owns; the task set never runs a queue of its own behind the host's back.
//! README.md records the scope and the boundary rules, including why the
//! `Waker` half of every poll context is inert: only `cx.local_waker()` is
//! wired, and **a leaf that parks `cx.waker()` never wakes**.

#![feature(local_waker)]

mod completion;
mod reactor;
mod runtime;
mod task_loop;
mod tasks;
mod timer;

pub use completion::{completion_pair, Completion, CompletionSender, WakeFn};
pub use reactor::{Interest, MioReactor, PollResult, Reactor, Readiness, Ready, Token};
pub use runtime::TaskRuntime;
pub use task_loop::TaskLoop;
pub use tasks::{
    join, sleep, sleep_until, yield_now, AsyncFd, Join, ReadinessFuture, Sleep, TaskEvent,
    TaskHandle, TaskId, TaskSet, WakeSink, YieldNow,
};
pub use timer::{ExpiredTimer, TimerId, TimerQueue};

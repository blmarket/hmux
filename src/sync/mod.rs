//! Async dataflow primitives the daemon composes over `hmux-rt`'s leaves.
//!
//! `hmux-rt` exposes runtime capability only: the reactor, the timer queue,
//! and the task machinery. Generic dataflow over those leaves — one-shot
//! completions, notification slots, racing and joining futures — needs none of
//! that access, so it belongs to the composer and lives here.
//!
//! Everything parks `LocalWaker`s, the same single-threaded contract as
//! `hmux-rt`: the `Waker` half of every poll context is inert, and **a leaf
//! that parks `cx.waker()` never wakes**.

mod completion;
mod future;
mod notify;

pub(crate) use completion::{completion_pair, Completion, CompletionSender, WakeFn};
pub(crate) use future::{join, select, yield_now, Either};
pub(crate) use notify::Notify;

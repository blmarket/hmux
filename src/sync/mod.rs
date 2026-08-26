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

pub(crate) use completion::{Completion, CompletionSender, WakeFn, completion_pair};
pub(crate) use future::{Either, join, maybe, race, select, yield_now};
pub(crate) use notify::Notify;

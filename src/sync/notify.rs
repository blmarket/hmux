//! Multi-shot notification between a producer and one parked task.
//!
//! A [`Completion`](super::Completion) hands over one value and is done; a
//! `Notify` is the recurring version of the same handoff with no value at all.
//! Notifying stores one permit and wakes the parked waiter; waiting takes the
//! permit. Notifies delivered while nobody waits collapse into that one
//! permit, which is the "drain until idle before waiting again" discipline
//! every consumer here follows anyway.
//!
//! One waiter at a time: a second `notified().await` on the same `Notify`
//! replaces the first waiter's waker. The single-consumer shape is the point —
//! a `Notify` belongs to one task and is signalled by everyone else.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, LocalWaker, Poll};

/// Cloneable handle to one notification slot.
#[derive(Clone, Default)]
pub struct Notify {
    shared: Rc<RefCell<NotifyState>>,
}

#[derive(Default)]
struct NotifyState {
    notified: bool,
    waker: Option<LocalWaker>,
}

impl Notify {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store the permit and wake the parked waiter, if any.
    ///
    /// Callable from anywhere on the loop's thread — inside another task,
    /// inside an actor's dispatch, or from host code between turns. The wake
    /// goes through the parked task's own waker, so the waiter resumes in the
    /// host's dispatch order, never inline.
    pub fn notify(&self) {
        let waker = {
            let mut state = self.shared.borrow_mut();
            state.notified = true;
            state.waker.take()
        };
        // Outside the borrow: the wake may reach a sink that turns around and
        // looks at this notify before returning.
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    /// Wait for the next permit, taking it.
    pub fn notified(&self) -> Notified {
        Notified {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub struct Notified {
    shared: Rc<RefCell<NotifyState>>,
}

impl Future for Notified {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        let mut state = self.shared.borrow_mut();
        if state.notified {
            state.notified = false;
            state.waker = None;
            return Poll::Ready(());
        }
        state.waker = Some(context.local_waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{completion_pair, yield_now};
    use hmux_rt::TaskRuntime;

    #[test]
    fn a_permit_stored_before_the_wait_resolves_it_at_once() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let notify = Notify::new();
        notify.notify();
        notify.notify();
        let waiter = notify.clone();
        runtime.block_on(async move {
            waiter.notified().await;
        });
        // Both notifies collapsed into the one permit the waiter took.
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let woken_again = runtime.block_on(async move {
            let (completion, sender) = completion_pair().expect("pair");
            let waiter = notify.clone();
            handle.spawn(async move {
                waiter.notified().await;
                sender.complete(true);
            });
            yield_now().await;
            notify.notify();
            completion.await.expect("waiter finished")
        });
        assert!(woken_again);
    }

    #[test]
    fn a_notify_wakes_the_parked_waiter() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let handle = runtime.handle();
        let notify = Notify::new();
        let signal = notify.clone();
        let observed = runtime.block_on(async move {
            let (completion, sender) = completion_pair().expect("pair");
            let waiter = notify.clone();
            handle.spawn(async move {
                waiter.notified().await;
                sender.complete(7u32);
            });
            // Give the waiter its first turn so it parks before the signal.
            yield_now().await;
            signal.notify();
            completion.await.expect("waiter finished")
        });
        assert_eq!(observed, 7);
    }
}

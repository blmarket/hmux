//! How a finished task hands its value back to whoever is waiting for it.
//!
//! This is runtime plumbing, not a primitive for tasks to compose with:
//! [`JoinHandle`] and [`TaskRuntime::block_on`] are the only waiters, and both
//! live inside this crate. Whoever composes over the leaves here builds its own
//! general-purpose version — one that carries no runtime access and so has no
//! reason to live in this crate.
//!
//! Both ends are on one thread, so the handoff is a shared slot and a callback
//! rather than anything the kernel knows about.
//!
//! [`JoinHandle`]: crate::JoinHandle
//! [`TaskRuntime::block_on`]: crate::TaskRuntime::block_on

use std::cell::RefCell;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// The producer stopped without producing a value: it was cancelled, or the
/// runtime holding it was dropped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stopped;

impl fmt::Display for Stopped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the producing task stopped without a value")
    }
}

/// The receiving end: the value a task will produce, and the wake that says it
/// has.
pub struct Handoff<T> {
    slot: Rc<RefCell<Slot<T>>>,
}

/// The producing end of a [`Handoff`].
///
/// Dropping it without a value closes the handoff, which the receiver reads as
/// [`Stopped`].
pub struct HandoffSender<T> {
    slot: Rc<RefCell<Slot<T>>>,
}

struct Slot<T> {
    value: Option<T>,
    closed: bool,
    /// Called once, by the producer, to tell whoever drives the parked
    /// receiver that its value has arrived.
    ///
    /// The callback belongs to the driver that installed it: it enqueues that
    /// driver's own wake event and returns. Resuming the receiver inline from
    /// here would make dispatch order depend on which producer happened to
    /// finish first, which is observable behavior.
    wake: Option<Rc<dyn Fn()>>,
}

pub fn handoff<T>() -> (Handoff<T>, HandoffSender<T>) {
    let slot = Rc::new(RefCell::new(Slot {
        value: None,
        closed: false,
        wake: None,
    }));
    (
        Handoff {
            slot: Rc::clone(&slot),
        },
        HandoffSender { slot },
    )
}

impl<T> HandoffSender<T> {
    pub fn complete(self, value: T) {
        let wake = {
            let mut slot = self.slot.borrow_mut();
            slot.value = Some(value);
            slot.wake.take()
        };
        // Outside the borrow: the wake reaches a driver that may look at this
        // very handoff before returning.
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl<T> Drop for HandoffSender<T> {
    fn drop(&mut self) {
        let wake = {
            let mut slot = self.slot.borrow_mut();
            if slot.value.is_some() {
                return;
            }
            slot.closed = true;
            slot.wake.take()
        };
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl<T> Handoff<T> {
    /// Whether the handoff has resolved, and with what.
    ///
    /// For a driver that is not a task and so has no waker to park with;
    /// a task awaits the [`Handoff`] itself.
    pub fn take(&mut self) -> Option<Result<T, Stopped>> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Some(Ok(value));
        }
        slot.closed.then_some(Err(Stopped))
    }

    /// Install the wake to call when the value arrives.
    ///
    /// A value that arrived before the driver got here fires the wake at once,
    /// so a handoff can never be installed onto too late. That is what lets a
    /// driver (re-)install on whatever schedule suits it instead of having to
    /// interleave with the producer.
    pub fn set_wake(&mut self, wake: &Rc<dyn Fn()>) {
        let mut slot = self.slot.borrow_mut();
        if slot.value.is_some() || slot.closed {
            drop(slot);
            wake();
            return;
        }
        slot.wake = Some(Rc::clone(wake));
    }
}

impl<T> Future for Handoff<T> {
    type Output = Result<T, Stopped>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Poll::Ready(Ok(value));
        }
        if slot.closed {
            return Poll::Ready(Err(Stopped));
        }
        let waker = context.local_waker().clone();
        slot.wake = Some(Rc::new(move || waker.wake_by_ref()));
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::handoff;
    use std::cell::Cell;
    use std::rc::Rc;

    /// Delivery and closure are covered by the runtime's own tests, which reach
    /// them through `block_on` and `JoinHandle`. What only `set_wake` has is
    /// install order, and neither order may leave the driver uninformed.
    fn counting_wake() -> (Rc<dyn Fn()>, Rc<Cell<u32>>) {
        let woken = Rc::new(Cell::new(0));
        let counter = Rc::clone(&woken);
        (Rc::new(move || counter.set(counter.get() + 1)), woken)
    }

    #[test]
    fn a_wake_installed_after_the_value_fires_at_once() {
        // A driver that reaches the handoff only after its producer finished
        // still gets told, so a late install can never wedge the waiter.
        let (mut handoff, sender) = handoff();
        let (wake, woken) = counting_wake();

        sender.complete(7);
        assert_eq!(woken.get(), 0, "no wake is installed yet");
        handoff.set_wake(&wake);
        assert_eq!(woken.get(), 1);
    }

    #[test]
    fn a_wake_installed_before_the_value_fires_on_completion() {
        let (mut handoff, sender) = handoff();
        let (wake, woken) = counting_wake();

        handoff.set_wake(&wake);
        assert_eq!(woken.get(), 0);
        sender.complete(7);
        assert_eq!(woken.get(), 1);
        assert_eq!(handoff.take().expect("the value"), Ok(7));
    }
}

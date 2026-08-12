//! The handoff between a task and whoever is waiting for what it produces.
//!
//! Both ends live on one thread, so this is a shared slot and a callback rather
//! than anything the kernel knows about.

use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Called once, by the producer, to tell whoever drives a parked task that its
/// value has arrived.
///
/// The callback belongs to the driver that installed it: it enqueues that
/// driver's own wake event and returns. Resuming the task inline from here
/// would make dispatch order depend on which producer happened to finish
/// first, which is observable behavior.
pub type WakeFn = Rc<dyn Fn()>;

/// The value a job the loop owns will produce, and the wake that says it has.
///
/// Both ends live on the loop, so the handoff is a shared slot rather than a
/// descriptor: the producer stores the value and calls the consumer's
/// [`WakeFn`]. Nothing here is pollable: there is no descriptor to register,
/// which is the whole point.
pub struct Completion<T> {
    slot: Rc<RefCell<CompletionSlot<T>>>,
}

/// The producing side of a [`Completion`].
///
/// Dropping it without a value closes the completion, which the consumer reads
/// as the work having stopped without a result.
pub struct CompletionSender<T> {
    slot: Rc<RefCell<CompletionSlot<T>>>,
}

struct CompletionSlot<T> {
    value: Option<T>,
    closed: bool,
    wake: Option<WakeFn>,
}

/// Fallible for its callers' sake only: nothing here can fail now that the
/// pair costs no descriptor.
pub fn completion_pair<T>() -> io::Result<(Completion<T>, CompletionSender<T>)> {
    let slot = Rc::new(RefCell::new(CompletionSlot {
        value: None,
        closed: false,
        wake: None,
    }));
    Ok((
        Completion {
            slot: Rc::clone(&slot),
        },
        CompletionSender { slot },
    ))
}

impl<T> CompletionSender<T> {
    pub fn complete(self, value: T) {
        let wake = {
            let mut slot = self.slot.borrow_mut();
            slot.value = Some(value);
            slot.wake.take()
        };
        // Outside the borrow: the wake reaches a driver that may look at this
        // very completion before returning.
        if let Some(wake) = wake {
            wake();
        }
    }
}

impl<T> Drop for CompletionSender<T> {
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

impl<T> Completion<T> {
    /// The value, if it has arrived, or the error a producer that stopped
    /// without one leaves behind.
    pub fn take(&mut self) -> Option<io::Result<T>> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Some(Ok(value));
        }
        slot.closed.then(|| {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "task worker stopped without a result",
            ))
        })
    }

    /// Install the wake to call when the value arrives.
    ///
    /// A value that arrived before the driver got here fires the wake at once,
    /// so a completion can never be installed onto too late. That is what lets
    /// a driver (re-)install on whatever schedule suits it instead of having to
    /// interleave with the producer.
    pub fn set_wake(&mut self, wake: &WakeFn) {
        let mut slot = self.slot.borrow_mut();
        if slot.value.is_some() || slot.closed {
            drop(slot);
            wake();
            return;
        }
        slot.wake = Some(Rc::clone(wake));
    }
}

impl<T> Future for Completion<T> {
    type Output = io::Result<T>;

    /// A task awaits its own completions; an owner that is not a task installs
    /// a [`WakeFn`] instead. Both are the same wait.
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Poll::Ready(Ok(value));
        }
        if slot.closed {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "task worker stopped without a result",
            )));
        }
        let waker = context.local_waker().clone();
        slot.wake = Some(Rc::new(move || waker.wake_by_ref()));
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::{completion_pair, WakeFn};
    use std::rc::Rc;

    #[test]
    fn completion_is_pending_until_its_sender_finishes() {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        assert!(completion.take().is_none());

        sender.complete(42);
        assert_eq!(completion.take().expect("the value").expect("no error"), 42);
    }

    #[test]
    fn a_dropped_sender_reports_work_that_stopped() {
        let (mut completion, sender) = completion_pair::<i32>().expect("completion pair");
        drop(sender);
        let error = completion
            .take()
            .expect("a closed completion resolves")
            .expect_err("with the producer's disappearance");
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn a_wake_installed_after_the_value_fires_at_once() {
        // The install-order race the fd-backed completion could not have: a
        // driver that reaches the completion only after its producer finished
        // still gets told, so a late install can never wedge the waiter.
        let (mut completion, sender) = completion_pair().expect("completion pair");
        let woken = Rc::new(std::cell::Cell::new(0));
        let counter = Rc::clone(&woken);
        let wake: WakeFn = Rc::new(move || counter.set(counter.get() + 1));

        sender.complete(7);
        assert_eq!(woken.get(), 0, "no wake is installed yet");
        completion.set_wake(&wake);
        assert_eq!(woken.get(), 1);
    }

    #[test]
    fn a_wake_installed_before_the_value_fires_on_completion() {
        let (mut completion, sender) = completion_pair().expect("completion pair");
        let woken = Rc::new(std::cell::Cell::new(0));
        let counter = Rc::clone(&woken);
        let wake: WakeFn = Rc::new(move || counter.set(counter.get() + 1));

        completion.set_wake(&wake);
        assert_eq!(woken.get(), 0);
        sender.complete(7);
        assert_eq!(woken.get(), 1);
        assert_eq!(completion.take().expect("the value").expect("no error"), 7);
    }
}

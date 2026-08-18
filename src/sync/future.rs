//! Racing, joining, and yielding: composition over futures that park
//! elsewhere.
//!
//! Nothing here reaches the runtime — every leaf being composed does its own
//! parking, and these combinators only forward the poll context. They live
//! with the composer for exactly that reason.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// The two outcomes of a [`select`].
pub enum Either<A, B> {
    First(A),
    Second(B),
}

/// Run two futures on one task, finishing when the first of them does.
///
/// Biased: `first` is polled before `second` on every turn, so a turn where
/// both are ready deterministically reports `first`. The loser is dropped with
/// the `Select`, which disarms whatever it parked — a lost [`hmux_rt::sleep`]
/// takes its deadline with it.
pub fn select<A: Future, B: Future>(first: A, second: B) -> Select<A, B> {
    Select { first, second }
}

pub struct Select<A, B> {
    first: A,
    second: B,
}

impl<A: Future, B: Future> Future for Select<A, B> {
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: `Select` is only ever pinned as a whole, and the projections
        // below are the standard structural ones.
        let this = unsafe { self.get_unchecked_mut() };
        if let Poll::Ready(output) = unsafe { Pin::new_unchecked(&mut this.first) }.poll(context) {
            return Poll::Ready(Either::First(output));
        }
        if let Poll::Ready(output) = unsafe { Pin::new_unchecked(&mut this.second) }.poll(context) {
            return Poll::Ready(Either::Second(output));
        }
        Poll::Pending
    }
}

/// Race a keyed set of futures on one task, reporting the key of the first to
/// finish along with what it produced.
///
/// [`select`] is the two-future version with the set known at compile time;
/// this is for a set that is computed — the sources one client is waiting on
/// this turn. Entries are polled in the order given, so an earlier one wins a
/// turn where both are ready, and an empty set is a future that never
/// finishes, which is what "nothing to wait on" means to a caller that races
/// this against something else.
///
/// Every entry is polled on every turn, so the futures raced have to be leaves
/// that park a fresh waker each poll rather than ones that only park on their
/// first — the runtime's readiness leaf is one. Dropping the race drops every
/// entry, which is what gives back whatever they took out.
pub fn race<K, F: Future + Unpin>(entries: Vec<(K, F)>) -> Race<K, F> {
    Race { entries }
}

pub struct Race<K, F> {
    entries: Vec<(K, F)>,
}

impl<K: Copy + Unpin, F: Future + Unpin> Future for Race<K, F> {
    type Output = (K, F::Output);

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        for (key, future) in self.get_mut().entries.iter_mut() {
            if let Poll::Ready(output) = Pin::new(future).poll(context) {
                return Poll::Ready((*key, output));
            }
        }
        Poll::Pending
    }
}

/// Wait on a future that may not be there, where absence means "never".
///
/// An optional deadline is the reason this exists: a client that has one races
/// it against its sources, and one that has none has to race those sources
/// against something, so `None` becomes the future that never finishes.
pub fn maybe<F: Future>(future: Option<F>) -> Maybe<F> {
    Maybe { future }
}

pub(crate) struct Maybe<F> {
    future: Option<F>,
}

impl<F: Future> Future for Maybe<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<F::Output> {
        // Safety: the projection is structural and nothing is moved out of the
        // pinned option.
        let this = unsafe { self.get_unchecked_mut() };
        match &mut this.future {
            Some(future) => unsafe { Pin::new_unchecked(future) }.poll(context),
            None => Poll::Pending,
        }
    }
}

/// Give the loop a turn before continuing.
///
/// A task that has more work but has used its budget parks itself behind
/// everything the loop already has queued, rather than running the loop's other
/// work late.
pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}

pub(crate) struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            return Poll::Ready(());
        }
        self.yielded = true;
        context.local_waker().wake_by_ref();
        Poll::Pending
    }
}

/// Run two futures on one task, finishing when both have.
///
/// The two pipes of a child process are the reason this exists: draining them
/// one after the other deadlocks as soon as the one not being read fills up.
pub fn join<A: Future, B: Future>(first: A, second: B) -> Join<A, B> {
    Join {
        first: Pending::Running(first),
        second: Pending::Running(second),
    }
}

enum Pending<F: Future> {
    Running(F),
    Done(Option<F::Output>),
}

impl<F: Future> Pending<F> {
    /// Poll unless already finished, and report the output once both halves
    /// can be taken.
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> bool {
        // Safety: the projection is structural and nothing here is moved out
        // of the pinned future while it is `Running`.
        let this = unsafe { self.get_unchecked_mut() };
        match this {
            Self::Running(future) => {
                let pinned = unsafe { Pin::new_unchecked(future) };
                match pinned.poll(context) {
                    Poll::Ready(output) => {
                        *this = Self::Done(Some(output));
                        true
                    }
                    Poll::Pending => false,
                }
            }
            Self::Done(_) => true,
        }
    }

    fn take(&mut self) -> F::Output {
        match self {
            Self::Done(output) => output.take().expect("a joined future reported twice"),
            Self::Running(_) => unreachable!("a joined future that has not finished"),
        }
    }
}

pub struct Join<A: Future, B: Future> {
    first: Pending<A>,
    second: Pending<B>,
}

impl<A: Future, B: Future> Future for Join<A, B> {
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: `Join` is only ever pinned as a whole, and the projections
        // below are the standard structural ones.
        let this = unsafe { self.get_unchecked_mut() };
        // Both halves are polled every time: whichever woke us is not knowable
        // here, and a poll of an already-finished half is free.
        let first = unsafe { Pin::new_unchecked(&mut this.first) }.poll(context);
        let second = unsafe { Pin::new_unchecked(&mut this.second) }.poll(context);
        if first && second {
            return Poll::Ready((this.first.take(), this.second.take()));
        }
        Poll::Pending
    }
}

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, LocalWaker, Poll};

#[derive(Clone, Default)]
pub struct Notify {
    state: Rc<RefCell<NotifyState>>,
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

    pub fn notify(&self) {
        let waker = {
            let mut state = self.state.borrow_mut();
            state.notified = true;
            state.waker.clone()
        };
        if let Some(waker) = waker {
            waker.wake_by_ref();
        }
    }

    pub fn notified(&self) -> Notified {
        Notified {
            state: Rc::clone(&self.state),
        }
    }
}

pub struct Notified {
    state: Rc<RefCell<NotifyState>>,
}

impl Future for Notified {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if state.notified {
            state.notified = false;
            state.waker = None;
            Poll::Ready(())
        } else {
            state.waker = Some(context.local_waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Notified {
    fn drop(&mut self) {
        self.state.borrow_mut().waker = None;
    }
}

/// Gives another task a chance to run before this task is polled again.
pub(crate) fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}

pub(crate) struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            context.local_waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub enum SelectResult<L, R> {
    Left(L),
    Right(R),
}

pub struct Select2<L, R> {
    left: L,
    right: R,
}

impl<L, R> Select2<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L, R> Future for Select2<L, R>
where
    L: Future,
    R: Future,
{
    type Output = SelectResult<L::Output, R::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let left = unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().left) };
        if let Poll::Ready(value) = left.poll(context) {
            return Poll::Ready(SelectResult::Left(value));
        }
        let right = unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().right) };
        if let Poll::Ready(value) = right.poll(context) {
            Poll::Ready(SelectResult::Right(value))
        } else {
            Poll::Pending
        }
    }
}

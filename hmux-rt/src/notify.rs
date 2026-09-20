//! Single-threaded wake and selection helpers. Notify supports one active waiter.
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
    next: u64,
    active: Option<u64>,
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
        let id = {
            let mut state = self.state.borrow_mut();
            state.next = state.next.checked_add(1).expect("waiter IDs exhausted");
            state.next
        };
        Notified {
            state: self.state.clone(),
            id,
        }
    }
}

pub struct Notified {
    state: Rc<RefCell<NotifyState>>,
    id: u64,
}

impl Future for Notified {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        assert!(
            state.active.is_none_or(|id| id == self.id),
            "Notify supports one active waiter"
        );
        if state.notified {
            state.notified = false;
            state.waker = None;
            state.active = None;
            Poll::Ready(())
        } else {
            state.active = Some(self.id);
            state.waker = Some(context.local_waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Notified {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        if state.active == Some(self.id) {
            state.waker = None;
            state.active = None;
        }
    }
}

/// Gives another task a chance to run before this task is polled again.
pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}

pub struct YieldNow {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskRuntime;
    use std::cell::Cell;
    #[test]
    fn wake_before_wait_coalesces_and_cancelled_wait_can_be_replaced() {
        let mut runtime = TaskRuntime::new().unwrap();
        let notify = Notify::new();
        notify.notify();
        notify.notify();
        runtime.block_on(notify.notified());
        let done = Rc::new(Cell::new(false));
        let flag = done.clone();
        let waiter = notify.clone();
        let task = runtime.handle().spawn(async move {
            waiter.notified().await;
            flag.set(true);
        });
        runtime.dispatch(8).unwrap();
        assert!(!done.get());
        runtime.handle().cancel(task);
        runtime.flush_cancelled().unwrap();
        notify.notify();
        runtime.block_on(notify.notified());
        assert!(!done.get());
    }
    #[test]
    fn dropping_unpolled_wait_does_not_remove_live_waiter() {
        let mut runtime = TaskRuntime::new().unwrap();
        let notify = Notify::new();
        let flag = Rc::new(Cell::new(false));
        let done = flag.clone();
        let waiter = notify.clone();
        runtime.handle().spawn(async move {
            waiter.notified().await;
            done.set(true);
        });
        runtime.dispatch(8).unwrap();
        drop(notify.notified());
        notify.notify();
        runtime.dispatch(8).unwrap();
        assert!(flag.get());
    }
    #[test]
    fn selection_drops_losing_wait_and_yield_allows_other_tasks() {
        let mut runtime = TaskRuntime::new().unwrap();
        let notify = Notify::new();
        let waiter = notify.clone();
        runtime.block_on(async move {
            assert!(matches!(
                Select2::new(waiter.notified(), yield_now()).await,
                SelectResult::Right(())
            ));
        });
        notify.notify();
        runtime.block_on(notify.notified());
    }
}

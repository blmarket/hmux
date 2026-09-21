use std::future::Future;
use std::io;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll};
use std::time::Instant;

use super::runtime::{Core, invalid};

/// Cancellation-safe absolute monotonic deadline wait.
pub struct Sleep {
    core: Weak<Core>,
    deadline: Instant,
    key: Option<(Instant, usize)>,
}

impl Sleep {
    pub(crate) fn new(core: &Rc<Core>, deadline: Instant) -> Self {
        Self {
            core: Rc::downgrade(core),
            deadline,
            key: None,
        }
    }
}

impl Future for Sleep {
    type Output = io::Result<()>;
    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let core = self.core.upgrade().ok_or_else(invalid)?;
        core.check()?;
        if Instant::now() >= self.deadline {
            if let Some(key) = self.key.take() {
                core.timers.borrow_mut().remove(&key);
            }
            return Poll::Ready(Ok(()));
        }
        let key = match self.key {
            Some(key) => key,
            None => {
                let key = (self.deadline, core.allocate()?);
                self.key = Some(key);
                key
            }
        };
        core.timers
            .borrow_mut()
            .insert(key, context.local_waker().clone());
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let (Some(core), Some(key)) = (self.core.upgrade(), self.key.take()) {
            core.timers.borrow_mut().remove(&key);
        }
    }
}

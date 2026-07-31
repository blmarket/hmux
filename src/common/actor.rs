//! Direct single-threaded actor references.

use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

/// Strong reference to actor state owned by a single-threaded event loop.
pub(crate) struct ActorRef<A> {
    inner: Rc<ActorCell<A>>,
}

struct ActorCell<A> {
    state: RefCell<Option<A>>,
}

impl<A> ActorRef<A> {
    pub(crate) fn new(actor: A) -> Self {
        Self {
            inner: Rc::new(ActorCell {
                state: RefCell::new(Some(actor)),
            }),
        }
    }

    pub(crate) fn downgrade(&self) -> WeakActorRef<A> {
        WeakActorRef {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn with<R>(&self, operation: impl FnOnce(&A) -> R) -> Option<R> {
        self.inner.state.borrow().as_ref().map(operation)
    }

    pub(crate) fn with_mut<R>(&self, operation: impl FnOnce(&mut A) -> R) -> Option<R> {
        self.inner.state.borrow_mut().as_mut().map(operation)
    }

    pub(crate) fn stop(&self) -> Option<A> {
        self.inner.state.borrow_mut().take()
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.inner.state.borrow().is_some()
    }
}

impl<A> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<A> fmt::Debug for ActorRef<A> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActorRef")
            .field("alive", &self.is_alive())
            .finish_non_exhaustive()
    }
}

/// Weak destination held by readiness registrations and long-lived sources.
pub(crate) struct WeakActorRef<A> {
    inner: Weak<ActorCell<A>>,
}

impl<A> WeakActorRef<A> {
    pub(crate) fn upgrade(&self) -> Option<ActorRef<A>> {
        let target = ActorRef {
            inner: self.inner.upgrade()?,
        };
        target.is_alive().then_some(target)
    }
}

impl<A> Clone for WeakActorRef<A> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

impl<A> fmt::Debug for WeakActorRef<A> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WeakActorRef")
            .field("allocated", &(self.inner.strong_count() != 0))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_reference_ignores_stopped_actor() {
        let actor = ActorRef::new(String::from("alive"));
        let weak = actor.downgrade();

        assert_eq!(
            weak.upgrade().and_then(|actor| actor.with(String::len)),
            Some(5)
        );
        assert_eq!(actor.stop().as_deref(), Some("alive"));
        assert!(weak.upgrade().is_none());
        assert!(!actor.is_alive());
    }

    #[test]
    fn queued_strong_reference_keeps_allocation_but_not_stopped_state() {
        let actor = ActorRef::new(1);
        let queued = actor.clone();
        drop(actor);

        assert_eq!(queued.with_mut(|value| *value += 1), Some(()));
        assert_eq!(queued.with(|value| *value), Some(2));
        assert_eq!(queued.stop(), Some(2));
        assert_eq!(queued.with(|value| *value), None);
    }
}

//! Readiness-driven Unix listener actor.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::rc::Rc;

use tracing::warn;

use crate::common::actor::ActorRef;
use crate::common::reactor::Token;

use super::driver::Outbox;

/// Accepted sockets waiting for the server adapter to create client actors.
#[derive(Clone, Default)]
pub(crate) struct AcceptedClients {
    inner: Rc<RefCell<VecDeque<UnixStream>>>,
}

impl AcceptedClients {
    pub(crate) fn pop_front(&self) -> Option<UnixStream> {
        self.inner.borrow_mut().pop_front()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    fn push_back(&self, stream: UnixStream) {
        self.inner.borrow_mut().push_back(stream);
    }
}

/// Events delivered to the listener actor.
pub(crate) enum ListenerEvent {
    Start,
    Readable,
    AcceptContinuation,
    Shutdown,
}

/// Listener ownership and bounded accept state.
pub(crate) struct Listener {
    listener: UnixListener,
    accepted: AcceptedClients,
    token: Option<Token>,
    accept_budget: usize,
    accept_work_queued: bool,
    stopping: bool,
}

impl Listener {
    pub(crate) fn new(listener: UnixListener, accept_budget: usize) -> (Self, AcceptedClients) {
        assert!(accept_budget > 0, "listener accept budget must be nonzero");
        let accepted = AcceptedClients::default();
        (
            Self {
                listener,
                accepted: accepted.clone(),
                token: None,
                accept_budget,
                accept_work_queued: false,
                stopping: false,
            },
            accepted,
        )
    }

    pub(crate) fn fd(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }

    pub(crate) fn token(&self) -> Option<Token> {
        self.token
    }

    pub(crate) fn set_token(&mut self, token: Option<Token>) {
        self.token = token;
    }

    pub(crate) fn mark_accept_work_queued(&mut self) -> bool {
        if self.stopping || self.accept_work_queued {
            return false;
        }
        self.accept_work_queued = true;
        true
    }

    pub(crate) fn request_shutdown(&mut self) -> bool {
        if self.stopping {
            return false;
        }
        self.stopping = true;
        true
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: ListenerEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            ListenerEvent::Start if !self.stopping => {
                outbox.set_listener_interest(target.clone(), true);
            }
            ListenerEvent::Readable | ListenerEvent::AcceptContinuation if !self.stopping => {
                self.accept_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ListenerEvent::Shutdown => {
                outbox.set_listener_interest(target.clone(), false);
                outbox.stop_listener(target.clone());
            }
            ListenerEvent::Start | ListenerEvent::Readable | ListenerEvent::AcceptContinuation => {}
        }
    }

    fn handle_readable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let mut accepted = 0;
        while accepted < self.accept_budget {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    self.accepted.push_back(stream);
                    accepted += 1;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    warn!(error = %error, "accept failed");
                    return;
                }
            }
        }

        if self.mark_accept_work_queued() {
            outbox.enqueue_listener(target.clone(), ListenerEvent::AcceptContinuation);
        }
    }
}

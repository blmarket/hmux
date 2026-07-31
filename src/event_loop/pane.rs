//! Reactor-owned pane PTY I/O.

use std::os::fd::BorrowedFd;

use crate::tmux::native::pane::PaneIo;

use super::actor::ActorRef;
use super::driver::Outbox;
use super::reactor::{Readiness, Token};

pub(crate) enum PaneEvent {
    Start,
    Ready(Readiness),
    ReadContinuation,
    Shutdown,
}

pub(crate) struct EventPane {
    io: PaneIo,
    token: Option<Token>,
    writable_interest: bool,
    work_queued: bool,
    stopping: bool,
}

impl EventPane {
    pub(crate) fn new(io: PaneIo) -> Self {
        Self {
            io,
            token: None,
            writable_interest: false,
            work_queued: false,
            stopping: false,
        }
    }

    pub(crate) fn fd(&self) -> BorrowedFd<'_> {
        self.io.as_fd()
    }

    pub(crate) fn token(&self) -> Option<Token> {
        self.token
    }

    pub(crate) fn set_token(&mut self, token: Option<Token>) {
        self.token = token;
    }

    pub(crate) fn mark_work_queued(&mut self) -> bool {
        if self.stopping || self.work_queued {
            return false;
        }
        self.work_queued = true;
        true
    }

    pub(crate) fn request_shutdown(&mut self) -> bool {
        if self.stopping {
            return false;
        }
        self.stopping = true;
        true
    }

    pub(crate) fn take_interest_change(&mut self) -> Option<bool> {
        if self.stopping {
            return None;
        }
        let writable = self.io.wants_write();
        if self.token.is_some() && writable == self.writable_interest {
            return None;
        }
        self.writable_interest = writable;
        Some(writable)
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: PaneEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            PaneEvent::Start if !self.stopping => self.sync_interest(target, outbox),
            PaneEvent::Ready(readiness) if !self.stopping => {
                self.work_queued = false;
                if readiness.is_writable() {
                    self.io.drive_writable();
                }
                if readiness.is_readable()
                    || readiness.is_read_closed()
                    || readiness.is_write_closed()
                    || readiness.is_error()
                {
                    match self.io.drive_readable() {
                        Ok(result) if result.closed => {
                            self.stopping = true;
                            outbox.set_pane_interest(target.clone(), false, false);
                            outbox.stop_pane(target.clone());
                            return;
                        }
                        Ok(result) if result.continuation => {
                            self.work_queued = true;
                            outbox.enqueue_pane(target.clone(), PaneEvent::ReadContinuation);
                        }
                        Ok(_) => {}
                        Err(_) => {
                            self.stopping = true;
                            outbox.set_pane_interest(target.clone(), false, false);
                            outbox.stop_pane(target.clone());
                            return;
                        }
                    }
                }
                self.sync_interest(target, outbox);
            }
            PaneEvent::ReadContinuation if !self.stopping => {
                self.work_queued = false;
                match self.io.drive_readable() {
                    Ok(result) if result.closed => {
                        self.stopping = true;
                        outbox.set_pane_interest(target.clone(), false, false);
                        outbox.stop_pane(target.clone());
                        return;
                    }
                    Ok(result) if result.continuation => {
                        self.work_queued = true;
                        outbox.enqueue_pane(target.clone(), PaneEvent::ReadContinuation);
                    }
                    Ok(_) => {}
                    Err(_) => {
                        self.stopping = true;
                        outbox.set_pane_interest(target.clone(), false, false);
                        outbox.stop_pane(target.clone());
                        return;
                    }
                }
                self.sync_interest(target, outbox);
            }
            PaneEvent::Shutdown => {
                self.stopping = true;
                outbox.set_pane_interest(target.clone(), false, false);
                outbox.stop_pane(target.clone());
            }
            PaneEvent::Start | PaneEvent::Ready(_) | PaneEvent::ReadContinuation => {}
        }
    }

    fn sync_interest(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if let Some(writable) = self.take_interest_change() {
            outbox.set_pane_interest(target.clone(), true, writable);
        }
    }
}

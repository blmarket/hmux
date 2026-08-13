//! Reactor-owned pane PTY I/O.

use std::os::fd::BorrowedFd;
use std::time::{Duration, Instant};

use crate::server::pane::PaneIo;

use super::actor::ActorRef;
use super::driver::Outbox;
use hmux_rt::{Readiness, Token};

/// tmux's ground timer: how long `input.c` waits for the terminator of a
/// string sequence before giving up on it and returning the parser to ground,
/// so a pane that opens an OSC and never closes it does not swallow everything
/// it writes afterwards.
const GROUND_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) enum PaneEvent {
    Start,
    Ready(Readiness),
    ReadContinuation,
    /// The ground timer expired on a string sequence still waiting for its
    /// terminator.
    GroundTimer(u64),
    Shutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaneInterest {
    Disabled,
    Readable,
    ReadableWritable,
}

pub(crate) struct EventPane {
    io: PaneIo,
    token: Option<Token>,
    writable_interest: bool,
    work_queued: bool,
    stopping: bool,
    /// Identifies the current ground timer so a sleep from an older parser
    /// state cannot expire a newer string sequence.
    timer_generation: u64,
    timer_armed: bool,
    /// Whether the parser was inside a string sequence at the end of the last
    /// read. tmux restarts the timer whenever such a sequence *begins*; this
    /// watches the same edge once per read, which against five seconds is not
    /// an observable difference.
    awaiting_terminator: bool,
}

impl EventPane {
    pub(crate) fn new(io: PaneIo) -> Self {
        Self {
            io,
            token: None,
            writable_interest: false,
            work_queued: false,
            stopping: false,
            timer_generation: 0,
            timer_armed: false,
            awaiting_terminator: false,
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

    pub(crate) fn take_interest_change(&mut self) -> Option<PaneInterest> {
        if self.stopping {
            return None;
        }
        let writable = self.io.wants_write();
        if self.token.is_some() && writable == self.writable_interest {
            return None;
        }
        self.writable_interest = writable;
        Some(if writable {
            PaneInterest::ReadableWritable
        } else {
            PaneInterest::Readable
        })
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: PaneEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            PaneEvent::Start if !self.stopping => self.sync_interest(target, outbox),
            PaneEvent::GroundTimer(generation)
                if !self.stopping && self.timer_armed && generation == self.timer_generation =>
            {
                self.timer_armed = false;
                self.awaiting_terminator = false;
                self.io.expire_ground();
                self.sync_interest(target, outbox);
            }
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
                            outbox.set_pane_interest(target.clone(), PaneInterest::Disabled);
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
                            outbox.set_pane_interest(target.clone(), PaneInterest::Disabled);
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
                        outbox.set_pane_interest(target.clone(), PaneInterest::Disabled);
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
                        outbox.set_pane_interest(target.clone(), PaneInterest::Disabled);
                        outbox.stop_pane(target.clone());
                        return;
                    }
                }
                self.sync_interest(target, outbox);
            }
            PaneEvent::Shutdown => {
                self.stopping = true;
                self.timer_armed = false;
                outbox.set_pane_interest(target.clone(), PaneInterest::Disabled);
                outbox.stop_pane(target.clone());
            }
            PaneEvent::Start
            | PaneEvent::Ready(_)
            | PaneEvent::ReadContinuation
            | PaneEvent::GroundTimer(_) => {}
        }
    }

    fn sync_interest(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        self.sync_ground_timer(target, outbox);
        if let Some(interest) = self.take_interest_change() {
            outbox.set_pane_interest(target.clone(), interest);
        }
    }

    /// Arm the ground timer when the pane has just entered a string sequence,
    /// and drop it when the sequence is over. `input_start_ground_timer` runs
    /// on the way into each of those states and `input_ground` cancels it, so
    /// the timer measures from where the sequence began rather than from the
    /// last byte of it.
    fn sync_ground_timer(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let awaiting = self.io.awaiting_terminator();
        if awaiting == self.awaiting_terminator {
            return;
        }
        self.awaiting_terminator = awaiting;
        if awaiting {
            self.timer_generation = self.timer_generation.wrapping_add(1);
            self.timer_armed = true;
            outbox.set_pane_timer(
                target.clone(),
                Instant::now() + GROUND_TIMEOUT,
                self.timer_generation,
            );
        } else {
            self.timer_armed = false;
        }
    }
}

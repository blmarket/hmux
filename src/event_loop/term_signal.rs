//! Readiness-driven `SIGINT`/`SIGTERM` teardown.
//!
//! Both signals write into a self-pipe the loop polls, so the shutdown decision
//! is made on the loop like every other event rather than by a thread parked in
//! `sigwait`. The signals stay deliverable process-wide: nothing blocks them, so
//! a child that never clears its mask is unaffected.

use std::io::{self, Read};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;

use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;
use tracing::info;

use super::actor::ActorRef;
use super::driver::Outbox;
use hmux_rt::Token;

pub(crate) enum TermSignalEvent {
    Start,
    Readable,
}

pub(crate) struct TermSignal {
    reader: UnixStream,
    registrations: Vec<SigId>,
    token: Option<Token>,
    work_queued: bool,
}

impl TermSignal {
    pub(crate) fn new() -> io::Result<Self> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        let mut registrations = Vec::new();
        for signal in [SIGINT, SIGTERM] {
            let writer = writer.try_clone()?;
            match pipe::register(signal, writer) {
                Ok(registration) => registrations.push(registration),
                Err(error) => {
                    for registration in registrations {
                        unregister(registration);
                    }
                    return Err(error);
                }
            }
        }
        Ok(Self {
            reader,
            registrations,
            token: None,
            work_queued: false,
        })
    }

    pub(crate) fn fd(&self) -> BorrowedFd<'_> {
        self.reader.as_fd()
    }

    pub(crate) fn token(&self) -> Option<Token> {
        self.token
    }

    pub(crate) fn set_token(&mut self, token: Option<Token>) {
        self.token = token;
    }

    pub(crate) fn mark_work_queued(&mut self) -> bool {
        if self.work_queued {
            return false;
        }
        self.work_queued = true;
        true
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: TermSignalEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            TermSignalEvent::Start => {
                outbox.set_term_signal_interest(target.clone(), true);
            }
            TermSignalEvent::Readable => {
                self.work_queued = false;
                if !self.drain() {
                    return;
                }
                info!("shutting down");
                // Exactly what the `sigwait` teardown did, and what tmux does:
                // leave the socket pathname in place — it is only ever unlinked
                // when a new server binds it.
                std::process::exit(0);
            }
        }
    }

    /// Whether a signal actually arrived, as opposed to a spurious wakeup.
    fn drain(&mut self) -> bool {
        let mut bytes = [0u8; 64];
        let mut delivered = false;
        loop {
            match self.reader.read(&mut bytes) {
                Ok(0) => return delivered,
                Ok(_) => delivered = true,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return delivered,
            }
        }
    }
}

impl Drop for TermSignal {
    fn drop(&mut self) {
        for registration in std::mem::take(&mut self.registrations) {
            unregister(registration);
        }
    }
}

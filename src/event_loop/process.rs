//! Readiness-driven child-exit notification.

use std::io::{self, Read};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;

use signal_hook::consts::signal::SIGCHLD;
use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;

use crate::common::reactor::Token;
use crate::native::NativeServer;

use super::actor::ActorRef;
use super::driver::Outbox;

pub(crate) enum ChildSignalEvent {
    Start,
    Readable,
    Retry,
    Shutdown,
}

pub(crate) struct ChildSignal {
    reader: UnixStream,
    registration: SigId,
    server: NativeServer,
    token: Option<Token>,
    work_queued: bool,
    stopping: bool,
}

impl ChildSignal {
    pub(crate) fn new(server: NativeServer) -> io::Result<Self> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        let registration = pipe::register(SIGCHLD, writer)?;
        Ok(Self {
            reader,
            registration,
            server,
            token: None,
            work_queued: false,
            stopping: false,
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

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: ChildSignalEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            ChildSignalEvent::Start if !self.stopping => {
                outbox.set_child_signal_interest(target.clone(), true);
            }
            ChildSignalEvent::Readable if !self.stopping => {
                self.work_queued = false;
                self.drain();
                self.try_reap(target, outbox);
            }
            ChildSignalEvent::Retry if !self.stopping => {
                self.work_queued = false;
                self.try_reap(target, outbox);
            }
            ChildSignalEvent::Shutdown => {
                self.stopping = true;
                outbox.set_child_signal_interest(target.clone(), false);
                outbox.stop_child_signal(target.clone());
            }
            ChildSignalEvent::Start | ChildSignalEvent::Readable | ChildSignalEvent::Retry => {}
        }
    }

    fn drain(&mut self) {
        let mut bytes = [0u8; 64];
        loop {
            match self.reader.read(&mut bytes) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    fn try_reap(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.server.try_reap_event_children() {
            return;
        }
        self.work_queued = true;
        outbox.enqueue_child_signal(target.clone(), ChildSignalEvent::Retry);
    }
}

impl Drop for ChildSignal {
    fn drop(&mut self) {
        unregister(self.registration);
    }
}

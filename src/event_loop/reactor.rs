//! Readiness polling boundary for the server event loop.

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::ops::{BitOr, BitOrAssign};
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::time::Duration;

use mio::unix::SourceFd;
use mio::{Events, Poll, Waker};

const WAKE_TOKEN: mio::Token = mio::Token(0);
const FIRST_SOURCE_TOKEN: usize = 1;
const DEFAULT_EVENT_CAPACITY: usize = 1024;

/// Stable identity for one reactor registration.
///
/// Values are not reused during a reactor's lifetime, preventing stale kernel
/// events from being delivered to a newer registration.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Token(usize);

impl Token {
    pub(crate) fn as_usize(self) -> usize {
        self.0
    }
}

/// Readiness operations requested for a registered descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Interest(u8);

impl Interest {
    pub(crate) const READABLE: Self = Self(1 << 0);
    pub(crate) const WRITABLE: Self = Self(1 << 1);

    pub(crate) fn is_readable(self) -> bool {
        self.0 & Self::READABLE.0 != 0
    }

    pub(crate) fn is_writable(self) -> bool {
        self.0 & Self::WRITABLE.0 != 0
    }

    fn to_mio(self) -> mio::Interest {
        match (self.is_readable(), self.is_writable()) {
            (true, true) => mio::Interest::READABLE | mio::Interest::WRITABLE,
            (true, false) => mio::Interest::READABLE,
            (false, true) => mio::Interest::WRITABLE,
            (false, false) => unreachable!("Interest has no empty public value"),
        }
    }
}

impl BitOr for Interest {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Interest {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Readiness observed for a descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Readiness {
    readable: bool,
    writable: bool,
    read_closed: bool,
    write_closed: bool,
    error: bool,
}

impl Readiness {
    pub(crate) fn is_readable(self) -> bool {
        self.readable
    }

    pub(crate) fn is_writable(self) -> bool {
        self.writable
    }

    pub(crate) fn is_read_closed(self) -> bool {
        self.read_closed
    }

    pub(crate) fn is_write_closed(self) -> bool {
        self.write_closed
    }

    pub(crate) fn is_error(self) -> bool {
        self.error
    }

    fn from_mio(event: &mio::event::Event) -> Self {
        Self {
            readable: event.is_readable(),
            writable: event.is_writable(),
            read_closed: event.is_read_closed(),
            write_closed: event.is_write_closed(),
            error: event.is_error(),
        }
    }
}

/// One readiness notification addressed to its registered recipient.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Ready<R> {
    token: Token,
    recipient: R,
    readiness: Readiness,
}

impl<R> Ready<R> {
    pub(crate) fn token(&self) -> Token {
        self.token
    }

    pub(crate) fn recipient(&self) -> &R {
        &self.recipient
    }

    pub(crate) fn readiness(&self) -> Readiness {
        self.readiness
    }
}

/// Result metadata for one poll operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PollResult {
    woken: bool,
    ready_count: usize,
}

impl PollResult {
    pub(crate) fn was_woken(self) -> bool {
        self.woken
    }

    pub(crate) fn ready_count(self) -> usize {
        self.ready_count
    }
}

/// Cross-thread handle that interrupts a blocked reactor poll.
#[derive(Clone)]
pub(crate) struct WakeHandle {
    inner: Arc<Waker>,
}

impl fmt::Debug for WakeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("WakeHandle").finish_non_exhaustive()
    }
}

impl WakeHandle {
    pub(crate) fn wake(&self) -> io::Result<()> {
        self.inner.wake()
    }
}

/// Backend-independent operations needed by the server event loop.
///
/// The backend duplicates every registered descriptor so deferred
/// deregistration remains valid if an actor replaces its source while effects
/// are being applied. `poll` appends readiness notifications to `output`.
pub(crate) trait Reactor<R>
where
    R: Clone,
{
    fn register(
        &mut self,
        source: BorrowedFd<'_>,
        interest: Interest,
        recipient: R,
    ) -> io::Result<Token>;

    fn reregister(&mut self, token: Token, interest: Interest) -> io::Result<()>;

    fn deregister(&mut self, token: Token) -> io::Result<()>;

    fn poll(
        &mut self,
        timeout: Option<Duration>,
        output: &mut Vec<Ready<R>>,
    ) -> io::Result<PollResult>;

    fn wake_handle(&self) -> WakeHandle;
}

struct Registration<R> {
    fd: OwnedFd,
    recipient: R,
    interest: Interest,
}

/// `mio` readiness backend used on Linux and macOS.
pub(crate) struct MioReactor<R> {
    poll: Poll,
    events: Events,
    wake_handle: WakeHandle,
    registrations: HashMap<Token, Registration<R>>,
    next_token: usize,
}

impl<R> fmt::Debug for MioReactor<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MioReactor")
            .field("registration_count", &self.registrations.len())
            .field("next_token", &self.next_token)
            .finish_non_exhaustive()
    }
}

impl<R> MioReactor<R> {
    pub(crate) fn new() -> io::Result<Self> {
        Self::with_event_capacity(DEFAULT_EVENT_CAPACITY)
    }

    fn with_event_capacity(event_capacity: usize) -> io::Result<Self> {
        let poll = Poll::new()?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE_TOKEN)?);
        Ok(Self {
            poll,
            events: Events::with_capacity(event_capacity),
            wake_handle: WakeHandle { inner: waker },
            registrations: HashMap::new(),
            next_token: FIRST_SOURCE_TOKEN,
        })
    }

    fn allocate_token(&mut self) -> io::Result<Token> {
        if self.next_token == usize::MAX {
            return Err(io::Error::other("reactor token space exhausted"));
        }
        let token = Token(self.next_token);
        self.next_token += 1;
        Ok(token)
    }

    fn registration(&self, token: Token) -> io::Result<&Registration<R>> {
        self.registrations.get(&token).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("reactor token {} is not registered", token.as_usize()),
            )
        })
    }
}

impl<R> Reactor<R> for MioReactor<R>
where
    R: Clone,
{
    fn register(
        &mut self,
        source: BorrowedFd<'_>,
        interest: Interest,
        recipient: R,
    ) -> io::Result<Token> {
        let token = self.allocate_token()?;
        let duplicated = unsafe { libc::fcntl(source.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
        if duplicated < 0 {
            return Err(io::Error::last_os_error());
        }
        // `F_DUPFD_CLOEXEC` returned a new descriptor owned by the reactor.
        let fd = unsafe { OwnedFd::from_raw_fd(duplicated) };
        let raw_fd = fd.as_raw_fd();
        let mut source = SourceFd(&raw_fd);
        self.poll.registry().register(
            &mut source,
            mio::Token(token.as_usize()),
            interest.to_mio(),
        )?;
        self.registrations.insert(
            token,
            Registration {
                fd,
                recipient,
                interest,
            },
        );
        Ok(token)
    }

    fn reregister(&mut self, token: Token, interest: Interest) -> io::Result<()> {
        let raw_fd = self.registration(token)?.fd.as_raw_fd();
        let mut source = SourceFd(&raw_fd);
        self.poll.registry().reregister(
            &mut source,
            mio::Token(token.as_usize()),
            interest.to_mio(),
        )?;
        self.registrations
            .get_mut(&token)
            .expect("registration disappeared after lookup")
            .interest = interest;
        Ok(())
    }

    fn deregister(&mut self, token: Token) -> io::Result<()> {
        let raw_fd = self.registration(token)?.fd.as_raw_fd();
        let mut source = SourceFd(&raw_fd);
        self.poll.registry().deregister(&mut source)?;
        self.registrations.remove(&token);
        Ok(())
    }

    fn poll(
        &mut self,
        timeout: Option<Duration>,
        output: &mut Vec<Ready<R>>,
    ) -> io::Result<PollResult> {
        self.events.clear();
        loop {
            match self.poll.poll(&mut self.events, timeout) {
                Ok(()) => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }

        let output_start = output.len();
        let mut woken = false;
        for event in &self.events {
            if event.token() == WAKE_TOKEN {
                woken = true;
                continue;
            }

            let token = Token(event.token().0);
            let Some(registration) = self.registrations.get(&token) else {
                // Readiness may already have been queued when its source was
                // deregistered. Tokens are never reused, so it is safe to drop.
                continue;
            };
            output.push(Ready {
                token,
                recipient: registration.recipient.clone(),
                readiness: Readiness::from_mio(event),
            });
        }

        Ok(PollResult {
            woken,
            ready_count: output.len() - output_start,
        })
    }

    fn wake_handle(&self) -> WakeHandle {
        self.wake_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::tmux::codec::{ImsgReader, ImsgWriter};
    use crate::tmux::message::{Frame, Message};
    use std::io::Write as _;
    use std::os::fd::AsFd as _;
    use std::os::unix::net::UnixStream;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Recipient {
        Client(u32),
    }

    fn socket_pair() -> (UnixStream, UnixStream) {
        let (sender, receiver) = UnixStream::pair().expect("socket pair");
        sender.set_nonblocking(true).expect("nonblocking sender");
        receiver
            .set_nonblocking(true)
            .expect("nonblocking receiver");
        (sender, receiver)
    }

    #[test]
    fn readable_descriptor_delivers_token_and_recipient() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (mut sender, receiver) = socket_pair();
        let recipient = Recipient::Client(7);
        let token = reactor
            .register(receiver.as_fd(), Interest::READABLE, recipient.clone())
            .expect("register");

        sender.write_all(b"x").expect("make socket readable");
        let mut ready = Vec::new();
        let result = reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("poll");

        assert_eq!(result.ready_count(), 1);
        assert!(!result.was_woken());
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].token(), token);
        assert_eq!(ready[0].recipient(), &recipient);
        assert!(ready[0].readiness().is_readable());
    }

    #[test]
    fn reregister_changes_interest_without_changing_token() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (_peer, source) = socket_pair();
        let token = reactor
            .register(source.as_fd(), Interest::READABLE, Recipient::Client(1))
            .expect("register");

        reactor
            .reregister(token, Interest::WRITABLE)
            .expect("reregister");
        let mut ready = Vec::new();
        reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("poll");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].token(), token);
        assert!(ready[0].readiness().is_writable());
    }

    #[test]
    fn deregister_remains_valid_after_actor_drops_its_source() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (_peer, source) = socket_pair();
        let token = reactor
            .register(source.as_fd(), Interest::READABLE, Recipient::Client(1))
            .expect("register");

        drop(source);

        reactor
            .deregister(token)
            .expect("deferred deregistration owns its descriptor");
    }

    #[test]
    fn deregistering_duplicated_write_fd_preserves_read_readiness() {
        use std::io::{Read as _, Write as _};
        use std::os::fd::{AsFd as _, OwnedFd};

        let mut reactor = MioReactor::new().expect("reactor");
        let (client, mut peer) = UnixStream::pair().expect("socket pair");
        let read_fd: OwnedFd = client.into();
        let write_fd = read_fd.as_fd().try_clone_to_owned().expect("dup");
        let read_token = reactor
            .register(read_fd.as_fd(), Interest::READABLE, Recipient::Client(1))
            .expect("register read");
        let write_token = reactor
            .register(write_fd.as_fd(), Interest::WRITABLE, Recipient::Client(2))
            .expect("register write");

        let mut ready = Vec::new();
        reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("initial writable poll");
        assert!(ready.iter().any(|event| event.token() == write_token));
        reactor.deregister(write_token).expect("deregister write");

        let request_bytes = b"request";
        let written = unsafe {
            libc::write(
                write_fd.as_raw_fd(),
                request_bytes.as_ptr().cast(),
                request_bytes.len(),
            )
        };
        assert_eq!(written, request_bytes.len() as isize);
        let mut request = [0; 7];
        peer.read_exact(&mut request).expect("receive request");
        peer.write_all(b"response").expect("send response");

        ready.clear();
        reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("response poll");
        assert!(
            ready
                .iter()
                .any(|event| event.token() == read_token && event.readiness().is_readable()),
            "read registration lost after duplicate write deregistration: {ready:?}"
        );
    }

    #[test]
    fn deregister_stops_delivery_and_unknown_tokens_are_errors() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (mut sender, receiver) = socket_pair();
        let token = reactor
            .register(receiver.as_fd(), Interest::READABLE, Recipient::Client(2))
            .expect("register");
        reactor.deregister(token).expect("deregister");

        sender.write_all(b"x").expect("write after deregister");
        let mut ready = Vec::new();
        // The byte is already in the socket buffer, so a registered fd would
        // report readable on a non-blocking poll. Waiting would only slow the
        // check down without making the absence of an event any stronger.
        let result = reactor
            .poll(Some(Duration::ZERO), &mut ready)
            .expect("poll");
        assert_eq!(result.ready_count(), 0);
        assert!(ready.is_empty());

        let error = reactor
            .reregister(token, Interest::READABLE)
            .expect_err("token was removed");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        let error = reactor
            .deregister(token)
            .expect_err("token is still removed");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn wake_handle_interrupts_a_blocked_poll() {
        let mut reactor = MioReactor::<Recipient>::new().expect("reactor");
        let wake_handle = reactor.wake_handle();
        // The wake is latched, not edge-triggered on an already-blocked poll, so
        // the worker does not have to be raced into the window where `poll` is
        // sleeping: a wake that lands first still returns the next `poll`
        // immediately. That keeps the check free of a timing delay.
        let wake_thread = thread::spawn(move || {
            wake_handle.wake().expect("wake reactor");
        });

        let started = Instant::now();
        let mut ready = Vec::new();
        let result = reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("poll");
        wake_thread.join().expect("wake thread");

        assert!(result.was_woken());
        assert_eq!(result.ready_count(), 0);
        assert!(ready.is_empty());
        assert!(started.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn poll_appends_to_the_callers_output() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (mut first_sender, first_receiver) = socket_pair();
        let (mut second_sender, second_receiver) = socket_pair();
        let first_token = reactor
            .register(
                first_receiver.as_fd(),
                Interest::READABLE,
                Recipient::Client(1),
            )
            .expect("register first");
        let second_token = reactor
            .register(
                second_receiver.as_fd(),
                Interest::READABLE,
                Recipient::Client(2),
            )
            .expect("register second");
        first_sender.write_all(b"a").expect("write first");
        second_sender.write_all(b"b").expect("write second");

        let mut ready = vec![Ready {
            token: Token(999),
            recipient: Recipient::Client(999),
            readiness: Readiness::default(),
        }];
        let result = reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("poll");

        assert_eq!(result.ready_count(), 2);
        assert_eq!(ready.len(), 3);
        let mut delivered = ready[1..]
            .iter()
            .map(|event| event.token())
            .collect::<Vec<_>>();
        delivered.sort();
        assert_eq!(delivered, vec![first_token, second_token]);
    }

    #[test]
    fn existing_imsg_descriptor_is_readiness_driven() {
        let mut reactor = MioReactor::new().expect("reactor");
        let (sender, receiver) = socket_pair();
        let mut reader = ImsgReader::new(receiver.into());
        let mut writer = ImsgWriter::new(sender.into());
        let token = reactor
            .register(reader.as_fd(), Interest::READABLE, Recipient::Client(42))
            .expect("register imsg reader");
        writer
            .send(Frame::new(Message::Command(vec!["list-sessions".into()])))
            .expect("send frame");

        let mut ready = Vec::new();
        reactor
            .poll(Some(Duration::from_secs(1)), &mut ready)
            .expect("poll");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].token(), token);
        let frame = reader.try_recv().expect("receive ready frame");
        assert_eq!(frame.msg, Message::Command(vec!["list-sessions".into()]));
    }
}

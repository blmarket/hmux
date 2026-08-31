//! The [`Reactor`] contract, exercised only through the trait.
//!
//! Each test builds one concrete backend and then hands it to a function that
//! knows nothing but `impl Reactor<Recipient>`, so what is asserted is the
//! contract a backend has to meet rather than the one `mio` happens to
//! provide. A second backend is covered by all of this the moment it is named
//! at the bottom of the file.
//!
//! This module sits beside [`crate::reactor`] rather than inside it, so the
//! only things it can reach are the ones the contract is made of.
//! `PollResult::ready_count` is not among them, and that is the point:
//! readiness is observed through the `output` a caller passes to `poll`,
//! which is what a backend actually promises to fill.

use std::io;
use std::io::Write as _;
use std::os::fd::{AsFd as _, OwnedFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::reactor::{Interest, MioReactor, Reactor, Token};

/// How long a test waits for readiness it expects to arrive.
const WAIT: Option<Duration> = Some(Duration::from_secs(1));

/// Long enough to show readiness does not arrive. Anything a test has already
/// written is in the socket buffer by the time it polls, so a registered
/// descriptor would report at once and waiting would only slow the check.
const NO_WAIT: Option<Duration> = Some(Duration::ZERO);

/// Who a registration belongs to, as far as these tests care.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Recipient {
    Client(u32),
}

/// A connected pair, both ends non-blocking.
fn socket_pair() -> (UnixStream, UnixStream) {
    let (sender, receiver) = UnixStream::pair().expect("socket pair");
    sender.set_nonblocking(true).expect("nonblocking sender");
    receiver.set_nonblocking(true).expect("nonblocking receiver");
    (sender, receiver)
}

/// A readable descriptor is delivered as its own token, the recipient it was
/// registered with, and a readiness that says readable.
fn readable_delivers_its_token_and_recipient(reactor: &mut impl Reactor<Recipient>) {
    let (mut sender, receiver) = socket_pair();
    let recipient = Recipient::Client(7);
    let token = reactor
        .register(receiver.as_fd(), Interest::READABLE, recipient.clone())
        .expect("register");

    sender.write_all(b"x").expect("make readable");
    let mut ready = Vec::new();
    reactor.poll(WAIT, &mut ready).expect("poll");

    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].token(), token);
    assert_eq!(ready[0].recipient(), &recipient);
    assert!(ready[0].readiness().is_readable());
}

/// Every registration gets a token of its own, so two are never confused for
/// each other.
fn each_registration_gets_its_own_token(reactor: &mut impl Reactor<Recipient>) {
    let (_first_peer, first) = socket_pair();
    let (_second_peer, second) = socket_pair();

    let first_token = reactor
        .register(first.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register first");
    let second_token = reactor
        .register(second.as_fd(), Interest::READABLE, Recipient::Client(2))
        .expect("register second");

    assert_ne!(first_token, second_token);
}

/// The same descriptor registered twice is two registrations, each with its
/// own token and recipient, and both are delivered.
fn one_descriptor_can_carry_two_registrations(reactor: &mut impl Reactor<Recipient>) {
    let (mut sender, receiver) = socket_pair();
    let first = reactor
        .register(receiver.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register first");
    let second = reactor
        .register(receiver.as_fd(), Interest::READABLE, Recipient::Client(2))
        .expect("register second");
    assert_ne!(first, second);

    sender.write_all(b"x").expect("make readable");
    let mut ready = Vec::new();
    reactor.poll(WAIT, &mut ready).expect("poll");

    let delivered: Vec<Token> = ready.iter().map(|event| event.token()).collect();
    assert!(delivered.contains(&first), "delivered: {delivered:?}");
    assert!(delivered.contains(&second), "delivered: {delivered:?}");
}

/// `poll` appends: whatever the caller already had in `output` is still there
/// afterwards, with the new readiness added to it.
fn poll_appends_to_what_the_caller_passed(reactor: &mut impl Reactor<Recipient>) {
    let (mut first_sender, first) = socket_pair();
    let (mut second_sender, second) = socket_pair();
    let first_token = reactor
        .register(first.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register first");
    reactor
        .register(second.as_fd(), Interest::READABLE, Recipient::Client(2))
        .expect("register second");

    first_sender.write_all(b"a").expect("write first");
    let mut ready = Vec::new();
    reactor.poll(WAIT, &mut ready).expect("first poll");
    let after_first = ready.len();
    assert!(after_first >= 1);

    second_sender.write_all(b"b").expect("write second");
    reactor.poll(WAIT, &mut ready).expect("second poll");

    assert!(ready.len() > after_first, "the second poll appended");
    assert_eq!(
        ready[0].token(),
        first_token,
        "what the caller already had is untouched"
    );
}

/// A deregistered descriptor stops being delivered, even with readiness
/// already waiting on it.
fn deregistering_stops_delivery(reactor: &mut impl Reactor<Recipient>) {
    let (mut sender, receiver) = socket_pair();
    let token = reactor
        .register(receiver.as_fd(), Interest::READABLE, Recipient::Client(2))
        .expect("register");
    reactor.deregister(token).expect("deregister");

    sender.write_all(b"x").expect("write after deregister");
    let mut ready = Vec::new();
    reactor.poll(NO_WAIT, &mut ready).expect("poll");

    assert!(ready.is_empty(), "delivered after deregister: {ready:?}");
}

/// A token the reactor does not know is an error, and so is one it has
/// already given up — the token is gone either way.
fn an_unknown_token_is_not_found(reactor: &mut impl Reactor<Recipient>) {
    let (_peer, receiver) = socket_pair();
    let token = reactor
        .register(receiver.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register");
    reactor.deregister(token).expect("deregister");

    let error = reactor
        .deregister(token)
        .expect_err("the token is already gone");
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

/// The backend duplicates every registered descriptor, so deregistration is
/// still valid after the actor has dropped the source it registered.
fn deregistering_outlives_the_source_it_was_given(reactor: &mut impl Reactor<Recipient>) {
    let (_peer, source) = socket_pair();
    let token = reactor
        .register(source.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register");

    drop(source);

    reactor
        .deregister(token)
        .expect("deferred deregistration owns its descriptor");
}

/// Deregistering one duplicate of a descriptor leaves the other registration
/// alone, which is what keeps a read watch alive when a write watch on the
/// same connection goes away.
fn deregistering_one_duplicate_leaves_the_other(reactor: &mut impl Reactor<Recipient>) {
    let (client, mut peer) = socket_pair();
    let read_fd: OwnedFd = client.into();
    let write_fd = read_fd.as_fd().try_clone_to_owned().expect("dup");

    let read_token = reactor
        .register(read_fd.as_fd(), Interest::READABLE, Recipient::Client(1))
        .expect("register read");
    let write_token = reactor
        .register(write_fd.as_fd(), Interest::WRITABLE, Recipient::Client(2))
        .expect("register write");

    let mut ready = Vec::new();
    reactor.poll(WAIT, &mut ready).expect("writable poll");
    assert!(ready.iter().any(|event| event.token() == write_token));
    reactor.deregister(write_token).expect("deregister write");

    peer.write_all(b"response").expect("send response");
    ready.clear();
    reactor.poll(WAIT, &mut ready).expect("readable poll");

    assert!(
        ready
            .iter()
            .any(|event| event.token() == read_token && event.readiness().is_readable()),
        "the read registration was lost: {ready:?}"
    );
}

/// A registration asking only to write is not delivered for reading.
fn an_interest_is_not_delivered_for_the_other_readiness(reactor: &mut impl Reactor<Recipient>) {
    let (mut sender, receiver) = socket_pair();
    reactor
        .register(receiver.as_fd(), Interest::WRITABLE, Recipient::Client(1))
        .expect("register");

    sender.write_all(b"x").expect("make readable");
    let mut ready = Vec::new();
    reactor.poll(NO_WAIT, &mut ready).expect("poll");

    assert!(
        ready.iter().all(|event| !event.readiness().is_readable()),
        "a write watch was told about a read: {ready:?}"
    );
}

/// Polling a reactor with nothing registered delivers nothing and is not an
/// error.
fn polling_an_empty_reactor_delivers_nothing(reactor: &mut impl Reactor<Recipient>) {
    let mut ready = Vec::new();
    reactor.poll(NO_WAIT, &mut ready).expect("poll");
    assert!(ready.is_empty());
}

/// Every contract above, against each backend this crate ships.
macro_rules! backend_contract {
    ($backend:ident, $new:expr, $($name:ident),+ $(,)?) => {
        mod $backend {
            use super::*;

            $(
                #[test]
                fn $name() {
                    let mut reactor = $new;
                    super::$name(&mut reactor);
                }
            )+
        }
    };
}

backend_contract!(
    mio,
    MioReactor::<Recipient>::new().expect("reactor"),
    readable_delivers_its_token_and_recipient,
    each_registration_gets_its_own_token,
    one_descriptor_can_carry_two_registrations,
    poll_appends_to_what_the_caller_passed,
    deregistering_stops_delivery,
    an_unknown_token_is_not_found,
    deregistering_outlives_the_source_it_was_given,
    deregistering_one_duplicate_leaves_the_other,
    an_interest_is_not_delivered_for_the_other_readiness,
    polling_an_empty_reactor_delivers_nothing,
);

//! The hmux listener and per-connection pairing loops.
//!
//! The generic [`TmuxServer`] path retains its blocking compatibility pumps.
//! The concrete native path forwards all accepted pairings on one readiness
//! loop. Both paths tap frames for introspection and preserve attached
//! `SCM_RIGHTS` descriptors.

use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tracing::{info, warn};

use crate::event_loop::driver::{EventLoop, PairingHandle};
use crate::event_loop::pairing::PairingCloseReason;
use crate::tmux::codec::{split_nonblocking_stream_with_queue_limit, split_stream, MAX_IMSGSIZE};
use crate::tmux::introspect::{Direction, LoggingReader, LoggingWriter};
use crate::tmux::native::NativeServer;
use crate::tmux::traits::{FrameReader, FrameWriter, TmuxServer};

type NativeEventLoop =
    EventLoop<crate::event_loop::reactor::MioReactor<crate::event_loop::driver::IoRecipient>>;
const FORWARDING_WRITE_QUEUE_LIMIT: usize = MAX_IMSGSIZE;

/// Bind `listen_path` and serve clients from `server`, forever.
pub fn run<T>(listen_path: &Path, server: T) -> io::Result<()>
where
    T: TmuxServer + 'static,
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
{
    run_until(listen_path, server, |_| false)
}

/// Bind `listen_path` and serve clients until `loop_done` returns true.
///
/// Keeping this callback separate from [`TmuxServer`] mirrors tmux's
/// `proc_loop(server_proc, server_loop)`: the protocol abstraction opens client
/// connections, while the concrete server runtime owns its lifecycle policy.
pub fn run_until<T, F>(listen_path: &Path, server: T, loop_done: F) -> io::Result<()>
where
    T: TmuxServer + 'static,
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
    F: Fn(&T) -> bool,
{
    run_with_handler(listen_path, server, loop_done, handle_client::<T>)
}

/// Bind `listen_path` and serve a concrete [`NativeServer`] through the
/// nonblocking event-loop forwarding adapter.
///
/// This remains a sibling of the established native listener path rather than
/// an optional capability on [`TmuxServer`].
pub fn run_event_loop(listen_path: &Path, server: NativeServer) -> io::Result<()> {
    const ACCEPT_BUDGET: usize = 64;
    const DISPATCH_BUDGET: usize = 256;
    const POLL_INTERVAL: Duration = Duration::from_millis(10);

    let listener = bind_listener(listen_path)?;
    let mut event_loop = EventLoop::new()?;
    let mut pairings = Vec::new();

    loop {
        if server.event_loop_shutdown_requested() {
            break;
        }
        let accept_budget_exhausted = accept_native_clients(
            &listener,
            &server,
            &mut event_loop,
            &mut pairings,
            ACCEPT_BUDGET,
        );
        event_loop.dispatch_with_budget(DISPATCH_BUDGET)?;
        reap_pairings(&mut pairings);
        if !accept_budget_exhausted && event_loop.pending_events() == 0 {
            event_loop.poll(Some(POLL_INTERVAL))?;
        }
    }

    // Match the compatibility listener's shutdown behavior: stop accepting,
    // then allow already accepted clients to finish their final handshake.
    drop(listener);
    while !pairings.is_empty() {
        event_loop.dispatch_with_budget(DISPATCH_BUDGET)?;
        reap_pairings(&mut pairings);
        if !pairings.is_empty() && event_loop.pending_events() == 0 {
            event_loop.poll(None)?;
        }
    }
    Ok(())
}

fn run_with_handler<T, F, H>(
    listen_path: &Path,
    server: T,
    loop_done: F,
    handle: H,
) -> io::Result<()>
where
    T: Send + Sync + 'static,
    F: Fn(&T) -> bool,
    H: Fn(UnixStream, &T) -> io::Result<()> + Copy + Send + 'static,
{
    let listener = bind_listener(listen_path)?;
    let server = Arc::new(server);
    let mut workers = Vec::new();
    loop {
        if loop_done(server.as_ref()) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                // On macOS/BSD accepted sockets inherit O_NONBLOCK from the
                // listener. The compatibility pumps require it cleared.
                if let Err(e) = stream.set_nonblocking(false) {
                    warn!(error = %e, "failed to set client socket blocking");
                    continue;
                }
                let server = Arc::clone(&server);
                workers.push(thread::spawn(move || {
                    if let Err(e) = handle(stream, server.as_ref()) {
                        warn!(error = %e, "client pairing ended");
                    }
                }));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => warn!(error = %e, "accept failed"),
        }

        let mut index = 0;
        while index < workers.len() {
            if workers[index].is_finished() {
                let worker = workers.swap_remove(index);
                let _ = worker.join();
            } else {
                index += 1;
            }
        }
    }

    // Do not let process shutdown cut off the final EXIT/EXITED handshake.
    drop(listener);
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

fn bind_listener(listen_path: &Path) -> io::Result<UnixListener> {
    // Remove a stale socket, but never unlink a live tmux/hmux listener. This is
    // especially important for the discoverable default path: unlinking a live
    // socket would strand the existing server and let two servers claim the
    // same pathname at different times.
    if listen_path.exists() && UnixStream::connect(listen_path).is_err() {
        let _ = std::fs::remove_file(listen_path);
    }
    let listener = UnixListener::bind(listen_path)?;
    listener.set_nonblocking(true)?;
    info!(socket = %listen_path.display(), "hmux listening");
    Ok(listener)
}

fn add_native_pairing(
    client: UnixStream,
    server: &NativeServer,
    event_loop: &mut NativeEventLoop,
) -> io::Result<PairingHandle> {
    let (client_reader, client_writer) =
        split_nonblocking_stream_with_queue_limit(client, FORWARDING_WRITE_QUEUE_LIMIT)?;
    let (server_reader, server_writer) =
        server.connect_nonblocking(FORWARDING_WRITE_QUEUE_LIMIT)?;
    Ok(event_loop.add_pairing(client_reader, client_writer, server_reader, server_writer))
}

fn accept_native_clients(
    listener: &UnixListener,
    server: &NativeServer,
    event_loop: &mut NativeEventLoop,
    pairings: &mut Vec<PairingHandle>,
    budget: usize,
) -> bool {
    for _ in 0..budget {
        match listener.accept() {
            Ok((stream, _)) => match add_native_pairing(stream, server, event_loop) {
                Ok(pairing) => pairings.push(pairing),
                Err(error) => warn!(error = %error, "failed to create native client pairing"),
            },
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return false,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                warn!(error = %error, "accept failed");
                return false;
            }
        }
    }
    true
}

fn reap_pairings(pairings: &mut Vec<PairingHandle>) {
    pairings.retain(|pairing| {
        if pairing.is_alive() {
            return true;
        }
        match pairing.status().close_reason() {
            Some(PairingCloseReason::PeerClosed | PairingCloseReason::Shutdown) => {}
            Some(PairingCloseReason::Error(kind)) => {
                warn!(?kind, "native client pairing ended with an I/O error");
            }
            Some(PairingCloseReason::FrameExceedsQueueLimit) => {
                warn!("native client frame exceeds forwarding queue limit");
            }
            None => warn!("native client pairing stopped without a close reason"),
        }
        false
    });
}

/// Pair one accepted client with a fresh server connection and pump frames.
fn handle_client<T: TmuxServer>(client: UnixStream, server: &T) -> io::Result<()>
where
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
{
    let (client_reader, client_writer) = split_stream(client)?;
    let (server_reader, server_writer) = server.connect()?;

    // Raw fd of the server socket, captured before the reader is wrapped/moved,
    // so the c2s pump can force it down when the client goes away (see below).
    let server_fd = server_reader.as_raw_fd();

    // Wrap each half with the introspection tap.
    let mut c2s_reader = LoggingReader::new(client_reader, Direction::ClientToServer);
    let mut c2s_writer = server_writer;
    let mut s2c_reader = LoggingReader::new(server_reader, Direction::ServerToClient);
    let mut s2c_writer = LoggingWriter::new(client_writer, Direction::ServerToClient);

    // client -> server on this thread's child; server -> client here.
    //
    // When the client -> server direction ends, the client's send side is gone:
    // it detached uncleanly, was killed, or the socket dropped. `split_stream`
    // gave each half an independent dup of the same socket, so the c2s pump
    // merely dropping its writer does *not* close the socket — the s2c half keeps
    // it open. On the interactive attach path the s2c pump then blocks forever
    // reading the server socket (the attach loop renders straight to the tty and
    // rarely sends control frames), and the attach loop itself spins on its
    // control read, never observing the disconnect and never restoring the
    // client's terminal out of raw mode. An explicit `shutdown` of the server
    // socket propagates the EOF to both the s2c pump and the connection handler,
    // so the attach loop breaks and puts the terminal back. It runs only once the
    // client is actually gone, so a clean detach/exit handshake (client alive
    // until it completes) is never cut short.
    let c2s = thread::spawn(move || {
        let result = pump(&mut c2s_reader, &mut c2s_writer);
        // SAFETY: `server_fd` is still owned by the s2c reader half; `shutdown`
        // only flips socket state and is safe to race with that pump's read,
        // which then observes EOF.
        unsafe {
            libc::shutdown(server_fd, libc::SHUT_RDWR);
        }
        result
    });
    let s2c_res = pump(&mut s2c_reader, &mut s2c_writer);

    let c2s_res = c2s.join().unwrap_or(Ok(()));
    // Surface the first non-EOF error, if any.
    first_real_error(c2s_res, s2c_res)
}

/// Copy frames from `reader` to `writer` until EOF or error.
fn pump<R: FrameReader, W: FrameWriter>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    loop {
        let frame = match reader.recv() {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        writer.send(frame)?;
    }
}

fn first_real_error(a: io::Result<()>, b: io::Result<()>) -> io::Result<()> {
    a?;
    b
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::tmux::codec::split_stream;
    use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};

    #[test]
    fn native_pairing_reaches_the_existing_protocol_handler() {
        let server = NativeServer::new().unwrap();
        let (peer, endpoint) = UnixStream::pair().unwrap();
        let (mut reader, mut writer) = split_stream(peer).unwrap();
        let mut event_loop = EventLoop::new().unwrap();
        let pairing = add_native_pairing(endpoint, &server, &mut event_loop).unwrap();

        let mut frame = Frame::new(Message::Command(vec!["list-sessions".into()]));
        frame.version = PROTOCOL_VERSION - 1;
        writer.send(frame).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let reply = loop {
            event_loop.dispatch_with_budget(256).unwrap();
            match reader.try_recv() {
                Ok(frame) => break frame,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to receive version response: {error}"),
            }
            assert!(Instant::now() < deadline, "timed out waiting for reply");
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };

        assert_eq!(reply.msg, Message::Version);
        drop(reader);
        drop(writer);
        while pairing.is_alive() && Instant::now() < deadline {
            event_loop.dispatch_with_budget(256).unwrap();
            if pairing.is_alive() && event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }
        assert!(!pairing.is_alive());
    }
}

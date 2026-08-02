//! The hmux listener and per-connection protocol loops.
//!
//! The generic [`TmuxServer`] path retains its blocking compatibility pumps.
//! The concrete event-loop path owns ordinary commands, control-mode clients,
//! interactive attach protocol routing, and pane PTY I/O. Both paths tap frames
//! for introspection and preserve attached `SCM_RIGHTS` descriptors.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tracing::{info, warn};

use crate::event_loop::driver::{EventLoop, ListenerHandle, PaneHandle, ProtocolHandle};
use crate::event_loop::protocol::ProtocolCloseReason;
use crate::server::Server;
use crate::tmux::codec::{split_nonblocking_stream_with_queue_limit, split_stream, MAX_IMSGSIZE};
use crate::tmux::introspect::{Direction, LoggingReader, LoggingWriter};
use crate::tmux::traits::{FrameReader, FrameWriter, TmuxServer};

type ReadinessEventLoop =
    EventLoop<crate::event_loop::reactor::MioReactor<crate::event_loop::driver::IoRecipient>>;
const PROTOCOL_WRITE_QUEUE_LIMIT: usize = MAX_IMSGSIZE;

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

/// Bind `listen_path` and serve a concrete [`Server`] through the
/// nonblocking event-loop protocol engine.
///
/// This remains a sibling of the established native listener path rather than
/// an optional capability on [`TmuxServer`].
pub fn run_event_loop(listen_path: &Path, server: Server) -> io::Result<()> {
    const ACCEPT_BUDGET: usize = 64;
    const DISPATCH_BUDGET: usize = 256;

    let mut event_loop = EventLoop::new()?;
    server.enable_event_loop_pane_io()?;
    let child_signal = event_loop.add_child_signal(server.clone())?;
    let listener = event_loop.add_listener(bind_listener(listen_path)?, ACCEPT_BUDGET)?;
    let mut clients = Vec::new();
    let mut panes = BTreeMap::new();

    loop {
        if server.event_loop_shutdown_requested() {
            break;
        }
        dispatch_event_loop_events(
            &server,
            &mut event_loop,
            &listener,
            &mut clients,
            DISPATCH_BUDGET,
        )?;
        server.reconcile_event_observations()?;
        sync_event_loop_panes(&server, &mut event_loop, &mut panes)?;
        reap_protocol_clients(&mut clients);
        if event_loop.pending_events() == 0 {
            event_loop.poll(None)?;
        }
    }

    // Match the compatibility listener's shutdown behavior: stop accepting,
    // then allow already accepted clients to finish their final handshake.
    event_loop.shutdown_listener(&listener);
    while listener.is_alive() {
        if !event_loop.dispatch_one()? {
            return Err(io::Error::other(
                "listener shutdown event disappeared before deregistration",
            ));
        }
        while listener.pop_accepted().is_some() {}
    }
    while !clients.is_empty() {
        event_loop.dispatch_with_budget(DISPATCH_BUDGET)?;
        server.reconcile_event_observations()?;
        sync_event_loop_panes(&server, &mut event_loop, &mut panes)?;
        reap_protocol_clients(&mut clients);
        if !clients.is_empty() && event_loop.pending_events() == 0 {
            event_loop.poll(None)?;
        }
    }
    for pane in panes.values() {
        event_loop.shutdown_pane(pane);
    }
    while panes.values().any(PaneHandle::is_alive) {
        if !event_loop.dispatch_one()? {
            return Err(io::Error::other(
                "pane shutdown event disappeared before deregistration",
            ));
        }
    }
    event_loop.shutdown_child_signal(&child_signal);
    while child_signal.is_alive() {
        if !event_loop.dispatch_one()? {
            return Err(io::Error::other(
                "child signal shutdown event disappeared before deregistration",
            ));
        }
    }
    Ok(())
}

fn sync_event_loop_panes(
    server: &Server,
    event_loop: &mut ReadinessEventLoop,
    panes: &mut BTreeMap<u64, PaneHandle>,
) -> io::Result<()> {
    let Some((new_panes, active_ids)) = server.try_event_pane_snapshot()? else {
        return Ok(());
    };
    for (runtime_id, io) in new_panes {
        if panes.contains_key(&runtime_id) {
            return Err(io::Error::other(format!(
                "duplicate pane runtime id {runtime_id}"
            )));
        }
        panes.insert(runtime_id, event_loop.add_pane(runtime_id, io));
    }

    let active_ids = active_ids.into_iter().collect::<BTreeSet<_>>();
    let removed = panes
        .iter()
        .filter_map(|(runtime_id, pane)| {
            (!active_ids.contains(runtime_id) || !pane.is_alive()).then_some(*runtime_id)
        })
        .collect::<Vec<_>>();
    for runtime_id in removed {
        if let Some(pane) = panes.remove(&runtime_id) {
            event_loop.shutdown_pane(&pane);
        }
    }
    for pane in panes.values() {
        event_loop.sync_pane(pane)?;
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
                        warn!(error = %e, "client connection ended");
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

fn add_protocol_client(
    client: UnixStream,
    server: &Server,
    event_loop: &mut ReadinessEventLoop,
) -> io::Result<ProtocolHandle> {
    let (reader, writer) =
        split_nonblocking_stream_with_queue_limit(client, PROTOCOL_WRITE_QUEUE_LIMIT)?;
    Ok(event_loop.add_protocol(reader, writer, server.clone()))
}

fn dispatch_event_loop_events(
    server: &Server,
    event_loop: &mut ReadinessEventLoop,
    listener: &ListenerHandle,
    clients: &mut Vec<ProtocolHandle>,
    budget: usize,
) -> io::Result<()> {
    for _ in 0..budget {
        if !event_loop.dispatch_one()? {
            break;
        }
        while let Some(stream) = listener.pop_accepted() {
            match add_protocol_client(stream, server, event_loop) {
                Ok(client) => clients.push(client),
                Err(error) => warn!(error = %error, "failed to create event-loop protocol client"),
            }
        }
    }
    Ok(())
}

fn reap_protocol_clients(clients: &mut Vec<ProtocolHandle>) {
    clients.retain(|client| {
        if client.is_alive() {
            return true;
        }
        match client.close_reason() {
            Some(
                ProtocolCloseReason::Completed
                | ProtocolCloseReason::PeerClosed
                | ProtocolCloseReason::Shutdown,
            ) => {}
            Some(ProtocolCloseReason::Error(kind)) => {
                warn!(?kind, "event-loop protocol client ended with an I/O error");
            }
            Some(ProtocolCloseReason::IdentifyExceedsLimit) => {
                warn!("client identification exceeds protocol limit");
            }
            Some(ProtocolCloseReason::FrameExceedsQueueLimit) => {
                warn!("protocol output frame exceeds queue limit");
            }
            None => warn!("event-loop protocol client stopped without a close reason"),
        }
        false
    });
}

/// Connect one accepted compatibility client and pump frames in both directions.
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
    use std::io::{Read, Write};
    use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::net::UnixStream;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::tmux::codec::{dup_fd, split_stream};
    use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};

    fn open_test_pty() -> (OwnedFd, OwnedFd) {
        let mut master = -1;
        let mut slave = -1;
        let size = libc::winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let result = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                &size,
            )
        };
        assert_eq!(result, 0, "openpty failed: {}", io::Error::last_os_error());
        let master = unsafe { OwnedFd::from_raw_fd(master) };
        let slave = unsafe { OwnedFd::from_raw_fd(slave) };
        let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) },
            0
        );
        (master, slave)
    }

    fn drain_fd(fd: &OwnedFd, output: &mut Vec<u8>) {
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let count = unsafe {
                libc::read(
                    fd.as_raw_fd(),
                    buffer.as_mut_ptr().cast::<libc::c_void>(),
                    buffer.len(),
                )
            };
            if count > 0 {
                output.extend_from_slice(&buffer[..count as usize]);
                continue;
            }
            if count < 0 {
                let error = io::Error::last_os_error();
                assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
            }
            break;
        }
    }

    #[test]
    fn event_loop_protocol_rejects_a_bad_version_directly() {
        let server = Server::new().unwrap();
        let (peer, endpoint) = UnixStream::pair().unwrap();
        let (mut reader, mut writer) = split_stream(peer).unwrap();
        let mut event_loop = EventLoop::new().unwrap();
        let client = add_protocol_client(endpoint, &server, &mut event_loop).unwrap();

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
        while client.is_alive() && Instant::now() < deadline {
            event_loop.dispatch_with_budget(256).unwrap();
            if client.is_alive() && event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }
        assert!(!client.is_alive());
    }

    #[test]
    fn event_loop_protocol_streams_command_response_across_high_water() {
        let server = Server::new().unwrap();
        let (peer, endpoint) = UnixStream::pair().unwrap();
        let (mut reader, mut writer) = split_stream(peer).unwrap();
        let (protocol_reader, protocol_writer) =
            split_nonblocking_stream_with_queue_limit(endpoint, 1).unwrap();
        let mut event_loop = EventLoop::new().unwrap();
        let client = event_loop.add_protocol(protocol_reader, protocol_writer, server.clone());

        writer
            .send(Frame::new(Message::Command(vec![
                "new-session".into(),
                "-d".into(),
                "-s".into(),
                "direct".into(),
                ";".into(),
                "list-sessions".into(),
                "-F".into(),
                "#{session_name}".into(),
            ])))
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(3);
        event_loop.dispatch_with_budget(256).unwrap();
        event_loop.poll(Some(Duration::from_secs(1))).unwrap();
        event_loop.dispatch_with_budget(256).unwrap();
        assert!(client.is_direct());

        let mut stdout = Vec::new();
        let mut exit_status = None;
        let exit = loop {
            event_loop.dispatch_with_budget(256).unwrap();
            loop {
                match reader.try_recv() {
                    Ok(frame) => match frame.msg {
                        Message::WriteOpen { stream, .. } => writer
                            .send(Frame::new(Message::WriteReady { stream, error: 0 }))
                            .unwrap(),
                        Message::Write { stream: 1, data } => stdout.extend_from_slice(&data),
                        Message::Exit(exit) => {
                            exit_status = exit;
                            break;
                        }
                        _ => {}
                    },
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error) => panic!("failed to receive command response: {error}"),
                }
            }
            if let Some(exit) = exit_status {
                break exit;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for command response"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };

        assert_eq!(exit, 0);
        assert_eq!(stdout, b"direct\n");
        drop((reader, writer));
        while client.is_alive() && Instant::now() < deadline {
            event_loop.dispatch_with_budget(256).unwrap();
            if client.is_alive() && event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }
        assert!(!client.is_alive());
    }

    #[test]
    fn event_loop_protocol_owns_control_mode_directly() {
        const CLIENT_CONTROL: i64 = 0x2000;

        let server = Server::new().unwrap();
        let (peer, endpoint) = UnixStream::pair().unwrap();
        let (mut reader, mut writer) = split_stream(peer).unwrap();
        let (mut control_input, passed_stdin) = UnixStream::pair().unwrap();
        let (mut control_output, passed_stdout) = UnixStream::pair().unwrap();
        control_output.set_nonblocking(true).unwrap();
        let mut event_loop = EventLoop::new().unwrap();
        let client = add_protocol_client(endpoint, &server, &mut event_loop).unwrap();

        writer
            .send(Frame::new(Message::IdentifyLongFlags(CLIENT_CONTROL)))
            .unwrap();
        writer
            .send(Frame::with_fd(Message::IdentifyStdin, passed_stdin.into()))
            .unwrap();
        writer
            .send(Frame::with_fd(
                Message::IdentifyStdout,
                passed_stdout.into(),
            ))
            .unwrap();
        writer.send(Frame::new(Message::IdentifyDone)).unwrap();
        writer
            .send(Frame::new(Message::Command(vec![
                "new-session".into(),
                "-A".into(),
                "-s".into(),
                "control".into(),
            ])))
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(3);
        while !client.is_control() {
            event_loop.dispatch_with_budget(256).unwrap();
            assert!(
                Instant::now() < deadline,
                "control client stayed identifying"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        control_input
            .write_all(b"display-message -p direct-control\n")
            .unwrap();
        let mut output = Vec::new();
        while !output
            .windows(b"direct-control\n".len())
            .any(|window| window == b"direct-control\n")
        {
            event_loop.dispatch_with_budget(256).unwrap();
            let mut buffer = [0u8; 4096];
            match control_output.read(&mut buffer) {
                Ok(count) => output.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to read control output: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for control output"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        drop(control_input);
        let exit = loop {
            event_loop.dispatch_with_budget(256).unwrap();
            match reader.try_recv() {
                Ok(frame) => {
                    if let Message::Exit(exit) = frame.msg {
                        break exit;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to receive control exit: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for control exit"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };
        assert_eq!(exit, Some(0));
    }

    #[test]
    fn event_loop_protocol_owns_interactive_attach_directly() {
        let server = Server::new().unwrap();
        let (peer, endpoint) = UnixStream::pair().unwrap();
        let (mut reader, mut writer) = split_stream(peer).unwrap();
        let (master, slave) = open_test_pty();
        let original_flags = unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFL) };
        let mut original_termios: libc::termios = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe { libc::tcgetattr(slave.as_raw_fd(), &mut original_termios) },
            0
        );
        let stdin = dup_fd(slave.as_fd()).unwrap();
        let stdout = dup_fd(slave.as_fd()).unwrap();
        let mut event_loop = EventLoop::new().unwrap();
        let client = add_protocol_client(endpoint, &server, &mut event_loop).unwrap();

        writer
            .send(Frame::new(Message::IdentifyTerm("event-loop-test".into())))
            .unwrap();
        writer
            .send(Frame::new(Message::IdentifyTerminfo(
                "clear=\x1b[2J".into(),
            )))
            .unwrap();
        writer
            .send(Frame::new(Message::IdentifyTerminfo(
                "cup=\x1b[%p1%d;%p2%dH".into(),
            )))
            .unwrap();
        writer
            .send(Frame::with_fd(Message::IdentifyStdin, stdin))
            .unwrap();
        writer
            .send(Frame::with_fd(Message::IdentifyStdout, stdout))
            .unwrap();
        writer.send(Frame::new(Message::IdentifyDone)).unwrap();
        writer
            .send(Frame::new(Message::Command(vec![
                "new-session".into(),
                "-s".into(),
                "direct-attach".into(),
            ])))
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut tty_output = Vec::new();
        let ready = loop {
            event_loop.dispatch_with_budget(512).unwrap();
            drain_fd(&master, &mut tty_output);
            match reader.try_recv() {
                Ok(frame) if frame.msg == Message::Ready => break true,
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to receive attach response: {error}"),
            }
            assert!(Instant::now() < deadline, "timed out starting attach");
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };
        assert!(ready);
        assert!(client.is_attach());

        let resized = libc::winsize {
            ws_row: 30,
            ws_col: 100,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        assert_eq!(
            unsafe { libc::ioctl(slave.as_raw_fd(), libc::TIOCSWINSZ, &resized) },
            0
        );
        writer.send(Frame::new(Message::Resize)).unwrap();
        loop {
            event_loop.dispatch_with_budget(512).unwrap();
            drain_fd(&master, &mut tty_output);
            let size = server
                .state()
                .lock()
                .unwrap()
                .sessions()
                .iter()
                .find(|session| session.name == "direct-attach")
                .map(|session| (session.cols, session.rows));
            if size == Some((100, 29)) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "timed out resizing attach client"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        let detach = b"\x02d";
        assert_eq!(
            unsafe {
                libc::write(
                    master.as_raw_fd(),
                    detach.as_ptr().cast::<libc::c_void>(),
                    detach.len(),
                )
            },
            detach.len() as isize
        );

        loop {
            event_loop.dispatch_with_budget(512).unwrap();
            drain_fd(&master, &mut tty_output);
            match reader.try_recv() {
                Ok(frame) => match frame.msg {
                    Message::Detach(Some(session)) => {
                        assert_eq!(session, "direct-attach");
                        writer.send(Frame::new(Message::Exiting)).unwrap();
                    }
                    Message::Exited => break,
                    _ => {}
                },
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed during attach detach: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "timed out detaching attach client"
            );
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        while client.is_alive() && Instant::now() < deadline {
            event_loop.dispatch_with_budget(512).unwrap();
            if event_loop.pending_events() == 0 {
                event_loop.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }
        assert!(!client.is_alive());
        assert!(!tty_output.is_empty());
        assert_eq!(
            unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFL) },
            original_flags
        );
        let mut restored_termios: libc::termios = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe { libc::tcgetattr(slave.as_raw_fd(), &mut restored_termios) },
            0
        );
        assert_eq!(restored_termios.c_iflag, original_termios.c_iflag);
        assert_eq!(restored_termios.c_oflag, original_termios.c_oflag);
        assert_eq!(restored_termios.c_cflag, original_termios.c_cflag);
        assert_eq!(restored_termios.c_lflag, original_termios.c_lflag);
        drop(slave);
    }
}

//! The hmux listener and per-connection protocol loops.
//!
//! The event-loop path owns ordinary commands, control-mode clients,
//! interactive attach protocol routing, and pane PTY I/O. Frames are tapped
//! for introspection and attached `SCM_RIGHTS` descriptors are preserved.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Instant;

use tracing::{info, warn};

use crate::event_loop::job::BackgroundRunner;
use crate::event_loop::listener::ListenerHandle;
use crate::event_loop::pane::PaneHandle;
use crate::event_loop::protocol::{self, ProtocolCloseReason, ProtocolHandle};
use crate::event_loop::{listener, pane, process, term_signal};
use crate::integration::AgentObserver;
use crate::platform::{CurrentPlatform, Platform};
use crate::server::Server;
use crate::tmux::codec::{split_nonblocking_stream_with_queue_limit, MAX_IMSGSIZE};
use hmux_rt::{TaskHandle, TaskRuntime};

const PROTOCOL_WRITE_QUEUE_LIMIT: usize = MAX_IMSGSIZE;

/// Bind `listen_path` and serve a concrete [`Server`] through the
/// nonblocking task-based protocol engine.
pub fn run_event_loop(
    listen_path: &Path,
    server: Server,
    mut observer: AgentObserver,
) -> io::Result<()> {
    const ACCEPT_BUDGET: usize = 64;
    const DISPATCH_BUDGET: usize = 256;

    let mut runtime = TaskRuntime::new()?;
    let tasks = runtime.handle();
    let child_signal = process::spawn(&tasks, server.clone())?;
    term_signal::spawn(&tasks)?;
    let listener = listener::spawn(&tasks, bind_listener(listen_path)?, ACCEPT_BUDGET)?;
    let background = BackgroundRunner::new(&server, tasks.clone());
    let mut clients = Vec::new();
    let mut panes = BTreeMap::new();
    let mut next_observer_tick = Instant::now();

    loop {
        if server.event_loop_shutdown_requested() {
            break;
        }
        runtime.dispatch(DISPATCH_BUDGET)?;
        adopt_accepted_clients(&tasks, &server, &background, &listener, &mut clients);
        server.reconcile_event_observations()?;
        let now = Instant::now();
        if now >= next_observer_tick {
            observer.tick(&server);
            next_observer_tick = now + AgentObserver::INTERVAL;
        }
        let timeout = earliest_timeout(
            earliest_timeout(server.refresh_alerts()?, server.refresh_lock_timers()?),
            Some(next_observer_tick.saturating_duration_since(Instant::now())),
        );
        sync_event_loop_panes(&tasks, &server, &mut panes)?;
        adopt_spawned_io(&tasks, &server);
        reap_protocol_clients(&mut clients);
        for request in server.enforce_lifecycle_policies()? {
            background.start(request);
        }
        if runtime.pending() == 0 {
            runtime.poll(timeout)?;
        }
    }

    // tmux saves the command-prompt history as its server loop exits.
    server.save_prompt_history();

    // Match the compatibility listener's shutdown behavior: stop accepting,
    // then allow already accepted clients to finish their final handshake.
    listener.shutdown();
    while listener.is_alive() {
        if runtime.dispatch(1)? == 0 {
            return Err(io::Error::other(
                "listener shutdown event disappeared before deregistration",
            ));
        }
        while listener.pop_accepted().is_some() {}
    }
    while !clients.is_empty() {
        runtime.dispatch(DISPATCH_BUDGET)?;
        server.reconcile_event_observations()?;
        sync_event_loop_panes(&tasks, &server, &mut panes)?;
        reap_protocol_clients(&mut clients);
        if !clients.is_empty() && runtime.pending() == 0 {
            runtime.poll(None)?;
        }
    }
    for pane in panes.values() {
        pane.shutdown();
    }
    while panes.values().any(PaneHandle::is_alive) {
        if runtime.dispatch(1)? == 0 {
            return Err(io::Error::other(
                "pane shutdown event disappeared before deregistration",
            ));
        }
    }
    child_signal.shutdown();
    while child_signal.is_alive() {
        if runtime.dispatch(1)? == 0 {
            return Err(io::Error::other(
                "child signal shutdown event disappeared before deregistration",
            ));
        }
    }
    Ok(())
}

/// The soonest of the server loop's own deadlines, or `None` when no timer is
/// armed and the loop may block until an event arrives.
fn earliest_timeout(
    left: Option<std::time::Duration>,
    right: Option<std::time::Duration>,
) -> Option<std::time::Duration> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, None) => left,
        (None, right) => right,
    }
}

/// Format `#()` jobs and `pipe-pane` children launched since the last pass.
/// Both are collected deep inside rendering and command handling, so the
/// server hands them over here and they run as tasks of their own.
fn adopt_spawned_io(tasks: &TaskHandle, server: &Server) {
    for job in server.take_pending_format_jobs() {
        let handle = tasks.clone();
        tasks.spawn(async move { job.run(&handle).await });
    }
    for pipe in server.take_new_pane_pipes() {
        let handle = tasks.clone();
        tasks.spawn(async move { pipe.run(&handle).await });
    }
}

fn sync_event_loop_panes(
    tasks: &TaskHandle,
    server: &Server,
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
        panes.insert(runtime_id, pane::spawn(tasks, io));
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
            pane.shutdown();
        }
    }
    for pane in panes.values() {
        pane.poke_write();
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
    tasks: &TaskHandle,
    server: &Server,
    background: &BackgroundRunner,
) -> io::Result<ProtocolHandle> {
    // The peer's uid is a property of the connection, so it is read while the
    // socket is still whole — tmux's `proc_add_peer` does the same at accept.
    let peer_uid = CurrentPlatform::peer_uid(client.as_fd());
    let (reader, writer) =
        split_nonblocking_stream_with_queue_limit(client, PROTOCOL_WRITE_QUEUE_LIMIT)?;
    Ok(protocol::spawn(
        tasks,
        reader,
        writer,
        server.clone(),
        background.clone(),
        peer_uid,
    ))
}

fn adopt_accepted_clients(
    tasks: &TaskHandle,
    server: &Server,
    background: &BackgroundRunner,
    listener: &ListenerHandle,
    clients: &mut Vec<ProtocolHandle>,
) {
    while let Some(stream) = listener.pop_accepted() {
        match add_protocol_client(stream, tasks, server, background) {
            Ok(client) => clients.push(client),
            Err(error) => warn!(error = %error, "failed to create event-loop protocol client"),
        }
    }
}

fn reap_protocol_clients(clients: &mut Vec<ProtocolHandle>) {
    clients.retain(|client| {
        if client.is_alive() {
            return true;
        }
        match client.close_reason() {
            Some(ProtocolCloseReason::Completed | ProtocolCloseReason::PeerClosed) => {}
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
        let mut runtime = TaskRuntime::new().unwrap();
        let background = BackgroundRunner::new(&server, runtime.handle());
        let client =
            add_protocol_client(endpoint, &runtime.handle(), &server, &background).unwrap();

        let mut frame = Frame::new(Message::Command(vec!["list-sessions".into()]));
        frame.version = PROTOCOL_VERSION - 1;
        writer.send(frame).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let reply = loop {
            runtime.dispatch(256).unwrap();
            match reader.try_recv() {
                Ok(frame) => break frame,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to receive version response: {error}"),
            }
            assert!(Instant::now() < deadline, "timed out waiting for reply");
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };

        assert_eq!(reply.msg, Message::Version);
        drop(reader);
        drop(writer);
        while client.is_alive() && Instant::now() < deadline {
            runtime.dispatch(256).unwrap();
            if client.is_alive() && runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
        let mut runtime = TaskRuntime::new().unwrap();
        let background = BackgroundRunner::new(&server, runtime.handle());
        let client = protocol::spawn(
            &runtime.handle(),
            protocol_reader,
            protocol_writer,
            server.clone(),
            background,
            None,
        );

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
        runtime.dispatch(256).unwrap();
        runtime.poll(Some(Duration::from_secs(1))).unwrap();
        runtime.dispatch(256).unwrap();
        assert!(client.is_direct());

        let mut stdout = Vec::new();
        let mut exit_status = None;
        let exit = loop {
            runtime.dispatch(256).unwrap();
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
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
            }
        };

        assert_eq!(exit, 0);
        assert_eq!(stdout, b"direct\n");
        drop((reader, writer));
        while client.is_alive() && Instant::now() < deadline {
            runtime.dispatch(256).unwrap();
            if client.is_alive() && runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
        let mut runtime = TaskRuntime::new().unwrap();
        let background = BackgroundRunner::new(&server, runtime.handle());
        let client =
            add_protocol_client(endpoint, &runtime.handle(), &server, &background).unwrap();

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
            runtime.dispatch(256).unwrap();
            assert!(
                Instant::now() < deadline,
                "control client stayed identifying"
            );
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
            runtime.dispatch(256).unwrap();
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
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        drop(control_input);
        let exit = loop {
            runtime.dispatch(256).unwrap();
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
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
        let mut runtime = TaskRuntime::new().unwrap();
        let background = BackgroundRunner::new(&server, runtime.handle());
        let client =
            add_protocol_client(endpoint, &runtime.handle(), &server, &background).unwrap();

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
            runtime.dispatch(512).unwrap();
            drain_fd(&master, &mut tty_output);
            match reader.try_recv() {
                Ok(frame) if frame.msg == Message::Ready => break true,
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("failed to receive attach response: {error}"),
            }
            assert!(Instant::now() < deadline, "timed out starting attach");
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
            runtime.dispatch(512).unwrap();
            drain_fd(&master, &mut tty_output);
            let size = server
                .state()
                .borrow_mut()
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
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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
            runtime.dispatch(512).unwrap();
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
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
            }
        }

        while client.is_alive() && Instant::now() < deadline {
            runtime.dispatch(512).unwrap();
            if runtime.pending() == 0 {
                runtime.poll(Some(Duration::from_millis(10))).unwrap();
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

//! Blocking readiness adapter for an interactive attach session.

use std::io;
use std::sync::{Arc, Mutex};

use crate::integration::status::StatusHub;
use crate::server::attach::{
    self, AttachDrive, AttachFrameReader, AttachPrepared, AttachWaitReady, AttachWaitSources,
    ClientTty,
};
use crate::server::command;
use crate::server::pane::PaneIoMode;
use crate::server::state::ServerState;
use crate::tmux::codec::ImsgReader;
use crate::tmux::message::{Frame, Message};
use crate::tmux::traits::FrameWriter;

impl AttachFrameReader for ImsgReader {
    fn has_buffered_frame(&self) -> bool {
        ImsgReader::has_buffered_frame(self)
    }

    fn try_recv(&mut self) -> io::Result<Frame> {
        ImsgReader::try_recv(self)
    }
}

pub(super) fn handle<R, W>(
    args: &[String],
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()>
where
    R: AttachFrameReader,
    W: FrameWriter,
{
    let mut session = match attach::start_attach_session(
        args,
        client_tty,
        state,
        hub,
        context,
        writer,
        PaneIoMode::Threaded(super::pane::spawn_reader),
    ) {
        Ok(session) => session,
        Err(failure) => return send_error_and_exit(reader, writer, &failure.into_message(), 1),
    };
    let control_fd = reader.as_raw_fd();

    loop {
        if run_deferred_command(&mut session, state, hub) {
            if session.drive_ready(state, hub, AttachWaitReady::default(), reader, writer)?
                == AttachDrive::Finished
            {
                break;
            }
            continue;
        }
        let ready = match session.prepare_wait(state, control_fd, reader.has_buffered_frame())? {
            AttachPrepared::Ready(ready) => ready,
            AttachPrepared::Wait { sources, timeout } => wait_for_events(sources, timeout)?,
            AttachPrepared::Finished => break,
        };
        if session.drive_ready(state, hub, ready, reader, writer)? == AttachDrive::Finished {
            break;
        }
    }
    Ok(())
}

fn run_deferred_command(
    session: &mut attach::AttachSession,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
) -> bool {
    let Some(request) = session.take_command_request() else {
        return false;
    };
    let agents = hub.snapshot().panes;
    let queue = match &request.source {
        command::DeferredCommand::Args(args) => {
            command::start_resumable_command(args, state, &agents, &request.context)
        }
        command::DeferredCommand::Line { line, tail } => {
            command::start_resumable_command_string_with_tail(
                line,
                tail,
                state,
                &agents,
                &request.context,
            )
        }
    };
    let mut result = match queue {
        Ok(queue) => super::task::NativeCommandRuntime::run(queue, state),
        Err(result) => result,
    };
    for background in result.background_commands.drain(..) {
        let _ = super::task::NativeCommandRuntime::spawn_background(
            background,
            Arc::clone(state),
            agents.clone(),
        );
    }
    session.complete_command(request.continuation, result, state);
    true
}

fn wait_for_events(sources: AttachWaitSources, timeout: i32) -> io::Result<AttachWaitReady> {
    let mut fds = [
        poll_read(sources.control),
        poll_read(sources.input),
        poll_write(sources.tty_output),
        poll_read(sources.output),
        poll_read(sources.prompt),
        poll_read(sources.render),
        poll_read(sources.status),
        poll_read(sources.popup_read),
        poll_write(sources.popup_write),
    ];
    let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout) };
    if result >= 0 {
        return Ok(AttachWaitReady {
            control: fds[0].revents != 0,
            tty_output: fds[2].revents != 0,
            output: fds[3].revents != 0,
            prompt: fds[4].revents != 0,
            render: fds[5].revents != 0,
            status: fds[6].revents != 0,
            popup_read: fds[7].revents != 0,
            popup_write: fds[8].revents != 0,
        });
    }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::Interrupted {
        return Ok(AttachWaitReady::default());
    }
    Err(error)
}

fn poll_read(fd: i32) -> libc::pollfd {
    libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    }
}

fn poll_write(fd: i32) -> libc::pollfd {
    libc::pollfd {
        fd,
        events: libc::POLLOUT,
        revents: 0,
    }
}

fn send_error_and_exit<R, W>(
    reader: &mut R,
    writer: &mut W,
    message: &str,
    exit_code: i32,
) -> io::Result<()>
where
    R: AttachFrameReader,
    W: FrameWriter,
{
    writer.send(Frame::new(Message::WriteOpen {
        stream: 2,
        fd: 2,
        flags: 0,
        path: Vec::new(),
    }))?;
    loop {
        match reader.recv() {
            Ok(frame) if matches!(frame.msg, Message::WriteReady { stream: 2, .. }) => break,
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(_) => break,
        }
    }
    writer.send(Frame::new(Message::Write {
        stream: 2,
        data: message.as_bytes().to_vec(),
    }))?;
    writer.send(Frame::new(Message::WriteClose { stream: 2 }))?;
    writer.send(Frame::new(Message::Exit(Some(exit_code))))
}

#[cfg(test)]
mod tests {
    use crate::integration::status::{AgentStatus, StatusHub};
    use crate::integration::AgentState;
    use crate::observability::v1::PaneId;
    use crate::server::attach::{AttachWaitReady, AttachWaitSources};

    use super::wait_for_events;

    #[test]
    fn agent_status_subscription_wakes_attach_poll_without_a_timer() {
        let hub = StatusHub::new();
        let subscription = hub.subscribe().expect("subscribe");
        subscription.drain();
        let sources = AttachWaitSources {
            control: -1,
            input: -1,
            tty_output: -1,
            output: -1,
            output_generation: 0,
            prompt: -1,
            render: -1,
            status: subscription.as_raw_fd(),
            popup_read: -1,
            popup_write: -1,
        };
        assert_eq!(
            wait_for_events(sources, 0).expect("poll"),
            AttachWaitReady::default()
        );

        hub.publish(
            PaneId(1),
            AgentStatus {
                agent: "codex",
                pid: Some(42),
                session_id: None,
                state: AgentState::Working,
            },
        );
        assert_eq!(
            wait_for_events(sources, 100).expect("poll"),
            AttachWaitReady {
                status: true,
                ..AttachWaitReady::default()
            }
        );
    }
}

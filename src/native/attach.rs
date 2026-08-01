//! Blocking readiness adapter for an interactive attach session.

use std::io;
use std::sync::{Arc, Mutex};

use crate::integration::status::StatusHub;
use crate::server::attach::{
    attach_target, explicit_target_session, send_error_and_exit, AttachDrive, AttachFrameReader,
    AttachPrepared, AttachSession, AttachStartFailure, AttachWaitReady, AttachWaitSources,
    ClientTty,
};
use crate::server::command;
use crate::server::state::ServerState;
use crate::tmux::codec::ImsgReader;
use crate::tmux::message::Frame;
use crate::tmux::traits::FrameWriter;

impl AttachFrameReader for ImsgReader {
    fn has_buffered_frame(&self) -> bool {
        ImsgReader::has_buffered_frame(self)
    }

    fn try_recv(&mut self) -> io::Result<Frame> {
        ImsgReader::try_recv(self)
    }
}

pub fn handle_attach<R, W>(
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
    let supplied_target = explicit_target_session(args);
    let target = {
        let mut st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        match attach_target(supplied_target, &mut st, context) {
            Ok(target) => target,
            Err(message) => {
                drop(st);
                return send_error_and_exit(reader, writer, &message, 1);
            }
        }
    };

    // Check session existence first, before tty checks, to match tmux ordering:
    // "can't find session" takes precedence over "not a terminal".
    {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        if st.find(&target).is_none() {
            let msg = format!("can't find session: {target}\n");
            drop(st);
            return send_error_and_exit(reader, writer, &msg, 1);
        }
    }

    run_attach(&target, client_tty, state, hub, context, reader, writer)
}

pub fn handle_new_session<R, W>(
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
    let target = {
        let mut st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        match command::new_session_for_attach(args, &mut st, context) {
            Ok(name) => name,
            Err(msg) => {
                drop(st);
                return send_error_and_exit(reader, writer, &msg, 1);
            }
        }
    };
    run_attach(&target, client_tty, state, hub, context, reader, writer)
}

fn run_attach<R, W>(
    target: &str,
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
    let mut session = match AttachSession::start(target, client_tty, state, hub, context, writer) {
        Ok(session) => session,
        Err(AttachStartFailure::Client(message)) => {
            return send_error_and_exit(reader, writer, &message, 1);
        }
        Err(AttachStartFailure::Io(error)) => return Err(error),
    };
    let imsg_fd = reader.as_raw_fd();

    // Main attach loop.
    loop {
        let ready = match session.prepare_wait(state, imsg_fd, reader.has_buffered_frame())? {
            AttachPrepared::Ready(ready) => ready,
            AttachPrepared::Wait { sources, timeout } => wait_for_attach_events(sources, timeout)?,
            AttachPrepared::Finished => break,
        };
        match session.drive_ready(state, hub, ready, reader, writer)? {
            AttachDrive::Continue => continue,
            AttachDrive::Finished => break,
        }
    }

    Ok(())
}

fn wait_for_attach_events(sources: AttachWaitSources, timeout: i32) -> io::Result<AttachWaitReady> {
    let mut fds = [
        libc::pollfd {
            fd: sources.control,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.input,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.tty_output,
            events: libc::POLLOUT,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.output,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.prompt,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.render,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.status,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.popup_read,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: sources.popup_write,
            events: libc::POLLOUT,
            revents: 0,
        },
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
        // Re-evaluate the absolute status deadline rather than restarting
        // the full relative poll timeout after every signal.
        return Ok(AttachWaitReady::default());
    }
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::status::StatusHub;

    #[test]
    fn agent_status_subscription_wakes_attach_poll_without_a_timer() {
        use crate::integration::status::AgentStatus;
        use crate::integration::AgentState;
        use crate::observability::v1::PaneId;

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
            wait_for_attach_events(sources, 0).expect("poll"),
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
            wait_for_attach_events(sources, 100).expect("poll"),
            AttachWaitReady {
                status: true,
                ..AttachWaitReady::default()
            }
        );
    }
}

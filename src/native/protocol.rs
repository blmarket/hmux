//! Per-connection control-plane state machine for the native server.
//!
//! Speaks the client side of tmux's imsg protocol well enough for a *command*
//! client (e.g. `tmux list-sessions`): version check → identify → command
//! dispatch, delivering output over the imsg file protocol
//! (`MSG_WRITE_OPEN`/`MSG_WRITE_READY`/`MSG_WRITE`/`MSG_WRITE_CLOSE`) and ending
//! with `MSG_EXIT`. This is the exact exchange the conformance suite asserts on.
//!
//! The interactive *attach* path (identify carries a tty fd; the server would
//! then composite panes onto it) is now implemented in `crate::server::attach`:
//! `attach-session` the handler takes the client's tty fd and drives it
//! directly via libghostty-vt, matching the real tmux flow where terminal I/O
//! bypasses imsg after identify.

use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::integration::status::{PaneAgents, StatusHub};
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::traits::{FrameReader, FrameWriter};

use crate::server::attach::{self, ClientTty};
use crate::server::command;
use crate::server::control::{EventControlClient, EventControlSource};
use crate::server::state::ServerState;

/// stdout / stderr fds the client maps opened streams onto.
const FD_STDOUT: i32 = 1;
const FD_STDERR: i32 = 2;

const CLIENT_CONTROL: i64 = 0x2000;

/// Handle one client connection to completion. Returns `Ok` on a clean end
/// (peer closed, command finished, or detach).
pub fn handle<R, W>(
    mut reader: R,
    mut writer: W,
    state: Arc<Mutex<ServerState>>,
    hub: StatusHub,
) -> io::Result<()>
where
    R: FrameReader + AsRawFd + attach::AttachFrameReader,
    W: FrameWriter,
{
    let mut client_tty = ClientTty::new();
    let mut client_context = command::ClientContext {
        wait_for_interactions: true,
        ..command::ClientContext::default()
    };
    let mut control_mode = false;

    loop {
        let frame = match reader.recv() {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };

        // Version is checked on every inbound frame, as tmux does
        // (proc.c `peer_check_version`): a bad-version peer gets MSG_VERSION and
        // is dropped.
        if frame.version != PROTOCOL_VERSION {
            writer.send(Frame::new(Message::Version))?;
            return Ok(());
        }

        match frame.msg {
            // Identify frames: accumulate client tty info for the attach path.
            // For the command path their details aren't needed and the passed
            // fds are dropped after being stored.
            Message::IdentifyFlags(flags) => {
                client_tty.flags |= i64::from(flags);
                control_mode |= i64::from(flags) & CLIENT_CONTROL != 0;
            }
            Message::IdentifyLongFlags(flags) => {
                client_tty.flags |= flags;
                control_mode |= flags & CLIENT_CONTROL != 0;
            }
            Message::IdentifyTerminfo(capability) => {
                client_tty.terminfo.push(capability);
            }
            Message::IdentifyFeatures(features) => {
                client_tty.features |= features as u32;
            }
            Message::IdentifyDone | Message::Resize => {
                /* accumulated identify data is consumed by the attach path */
            }
            Message::Flags(flags) => {
                control_mode |= flags & CLIENT_CONTROL != 0;
            }

            Message::IdentifyTerm(term) => {
                client_tty.term = Some(term);
            }
            Message::IdentifyTtyName(tty_name) => {
                client_context.tty_name = Some(tty_name.clone());
                client_tty.tty_name = Some(tty_name);
            }
            Message::IdentifyClientPid(pid) => {
                client_context.client_pid = Some(pid);
                client_tty.client_pid = Some(pid);
            }
            Message::IdentifyCwd(cwd) => {
                client_context.cwd = Some(cwd.into());
            }
            Message::IdentifyEnviron(entry) => {
                client_context.environment.push(entry);
            }
            Message::IdentifyStdin => {
                if let Some(fd) = frame.fd {
                    client_tty.stdin = Some(fd);
                }
            }
            Message::IdentifyStdout => {
                if let Some(fd) = frame.fd {
                    client_tty.stdout = Some(fd);
                }
            }

            Message::Command(args) => {
                if control_mode {
                    return handle_control_client(
                        &args,
                        client_tty,
                        &state,
                        &hub,
                        &client_context,
                        &mut reader,
                        &mut writer,
                    );
                }
                // A client carrying a real tty either attaches (interactive) or
                // runs a one-shot command, exactly as real tmux decides from the
                // command line (see [`command::classify`]).
                match command::classify(&args) {
                    command::Intent::Attach => {
                        return super::attach::handle_attach(
                            &args,
                            client_tty,
                            &state,
                            &hub,
                            &client_context,
                            &mut reader,
                            &mut writer,
                        );
                    }
                    command::Intent::NewAttach => {
                        return super::attach::handle_new_session(
                            &args,
                            client_tty,
                            &state,
                            &hub,
                            &client_context,
                            &mut reader,
                            &mut writer,
                        );
                    }
                    command::Intent::Command => {
                        dispatch_command(
                            &args,
                            &state,
                            &hub,
                            &client_context,
                            &mut reader,
                            &mut writer,
                        )?;
                        return Ok(());
                    }
                }
            }

            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) => return Ok(()),

            Message::Shutdown => return Ok(()),

            // Anything else on an inbound control connection: ignore and keep
            // going (mirrors tmux tolerating unexpected control frames).
            _ => {}
        }
    }
}

fn handle_control_client<R: FrameReader, W: FrameWriter>(
    args: &[String],
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    _reader: &mut R,
    writer: &mut W,
) -> io::Result<()> {
    let mut control =
        EventControlClient::new(args, client_tty, Arc::clone(state), hub.clone(), context)?;

    loop {
        control.drive(None)?;
        send_event_control_frames(&mut control, writer)?;
        if control.is_finished() {
            return Ok(());
        }

        let sources = control.sources();
        let mut pollfds = sources
            .iter()
            .map(|source| libc::pollfd {
                fd: control
                    .source_fd(*source)
                    .map(|fd| fd.as_raw_fd())
                    .unwrap_or(-1),
                events: if EventControlClient::source_is_writable(*source) {
                    libc::POLLOUT
                } else {
                    libc::POLLIN | libc::POLLHUP
                },
                revents: 0,
            })
            .collect::<Vec<_>>();
        let timeout = control.deadline(Instant::now()).map_or(-1, |deadline| {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                0
            } else {
                remaining
                    .as_nanos()
                    .saturating_add(999_999)
                    .checked_div(1_000_000)
                    .unwrap_or(u128::MAX)
                    .min(i32::MAX as u128) as i32
            }
        });
        let ready = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as _, timeout) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }

        for (source, pollfd) in sources.into_iter().zip(pollfds) {
            let mask = if EventControlClient::source_is_writable(source) {
                libc::POLLOUT
            } else {
                libc::POLLIN | libc::POLLHUP
            };
            if pollfd.revents & mask == 0 {
                continue;
            }
            loop {
                control.drive(Some(source))?;
                send_event_control_frames(&mut control, writer)?;
                if control.is_finished() {
                    return Ok(());
                }
                if source != EventControlSource::Input || !control.take_input_continuation() {
                    break;
                }
            }
        }
    }
}

fn send_event_control_frames<W: FrameWriter>(
    control: &mut EventControlClient,
    writer: &mut W,
) -> io::Result<()> {
    while let Some(frame) = control.pop_frame() {
        writer.send(frame)?;
    }
    Ok(())
}

/// Run a command and stream its output back, then send `MSG_EXIT`.
fn dispatch_command<R, W>(
    args: &[String],
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()>
where
    R: FrameReader,
    W: FrameWriter,
{
    // A `list-panes -F '#{pane_agent}'` reads pane agent status from the hub, so
    // snapshot it and thread it through command expansion.
    let agents = hub.snapshot().panes;
    let aliases = state
        .lock()
        .map_err(|_| io::Error::other("native server state poisoned"))?
        .command_aliases();
    let (line_groups, line_validation) = match command::command_line_groups(args, &aliases) {
        Ok(groups) => (groups, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    let result = if let Some(error) = line_validation {
        error
    } else if line_groups.len() > 1
        && line_groups
            .iter()
            .any(|group| command::uses_client_file_protocol(group))
    {
        dispatch_client_command_groups(&line_groups, state, &agents, context, reader, writer)?
    } else {
        dispatch_plain_command(args, state, &agents, context, reader, writer)?
    };

    // Streams are opened only when there's output, matching tmux (which opens a
    // client file lazily). stdout is stream 1, stderr stream 2.
    if !result.stdout_data().is_empty() {
        write_stream(reader, writer, 1, FD_STDOUT, result.stdout_data())?;
    }
    if !result.stderr.is_empty() {
        write_stream(reader, writer, 2, FD_STDERR, result.stderr.as_bytes())?;
    }

    writer.send(Frame::new(Message::Exit(Some(result.exit))))?;
    Ok(())
}

fn dispatch_client_command_groups<R, W>(
    groups: &[Vec<String>],
    state: &Arc<Mutex<ServerState>>,
    agents: &PaneAgents,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<command::CommandResult>
where
    R: FrameReader,
    W: FrameWriter,
{
    let mut output = command::CommandResult::ok("");
    for group in groups {
        let result = dispatch_plain_command(group, state, agents, context, reader, writer)?;
        let exit = result.exit;
        let continue_queue = result.continue_queue;
        output.continue_queue |= continue_queue;
        output.append_stdout(&result);
        output.stderr.push_str(&result.stderr);
        if output.exit == 0 || exit != 0 {
            output.exit = exit;
        }
        if exit != 0 && !continue_queue {
            break;
        }
    }
    Ok(output)
}

fn dispatch_plain_command<R, W>(
    args: &[String],
    state: &Arc<Mutex<ServerState>>,
    agents: &PaneAgents,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<command::CommandResult>
where
    R: FrameReader,
    W: FrameWriter,
{
    let file_write = {
        let state = state
            .lock()
            .map_err(|_| io::Error::other("native server state poisoned"))?;
        command::save_buffer_client_request(args, &state, context)
    };
    Ok(if let Some(request) = file_write {
        match request {
            Err(result) => result,
            Ok(request) => match write_client_file(reader, writer, &request)? {
                0 => command::CommandResult::ok(""),
                error => {
                    let mut result = command::CommandResult::err(format!(
                        "{}: {}\n",
                        io::Error::from_raw_os_error(error),
                        request.display_path
                    ));
                    result.continue_queue = true;
                    result
                }
            },
        }
    } else {
        let mut execution_context = context.clone();
        if let Some(path) = command::client_input_path(args, context) {
            execution_context.input_file = Some(read_client_file(reader, writer, &path)?);
        }
        command::run_with_context(args, state, agents, &execution_context)
    })
}

fn read_client_file<R, W>(
    reader: &mut R,
    writer: &mut W,
    path: &std::path::Path,
) -> io::Result<Result<Vec<u8>, i32>>
where
    R: FrameReader,
    W: FrameWriter,
{
    const STREAM: i32 = 3;
    let fd = if path.as_os_str() == "-" { 0 } else { -1 };
    let mut wire_path = path.as_os_str().as_bytes().to_vec();
    wire_path.push(0);
    writer.send(Frame::new(Message::ReadOpen {
        stream: STREAM,
        fd,
        path: wire_path,
    }))?;
    let mut data = Vec::new();
    loop {
        let frame = reader.recv()?;
        if frame.version != PROTOCOL_VERSION {
            writer.send(Frame::new(Message::Version))?;
            return Err(io::Error::other("client sent bad version during file read"));
        }
        match frame.msg {
            Message::Read {
                stream: STREAM,
                data: chunk,
            } => data.extend_from_slice(&chunk),
            Message::ReadDone {
                stream: STREAM,
                error,
            } => return Ok(if error == 0 { Ok(data) } else { Err(error) }),
            _ => {}
        }
    }
}

fn write_client_file<R, W>(
    reader: &mut R,
    writer: &mut W,
    request: &command::ClientFileWrite,
) -> io::Result<i32>
where
    R: FrameReader,
    W: FrameWriter,
{
    const STREAM: i32 = 3;
    let mut wire_path = request.path.as_os_str().as_bytes().to_vec();
    wire_path.push(0);
    writer.send(Frame::new(Message::WriteOpen {
        stream: STREAM,
        fd: -1,
        flags: request.flags,
        path: wire_path,
    }))?;
    loop {
        let frame = reader.recv()?;
        if frame.version != PROTOCOL_VERSION {
            writer.send(Frame::new(Message::Version))?;
            return Err(io::Error::other(
                "client sent bad version during file write",
            ));
        }
        if let Message::WriteReady {
            stream: STREAM,
            error,
        } = frame.msg
        {
            if error != 0 {
                return Ok(error);
            }
            break;
        }
    }
    for chunk in request.data.chunks(8 * 1024) {
        writer.send(Frame::new(Message::Write {
            stream: STREAM,
            data: chunk.to_vec(),
        }))?;
    }
    writer.send(Frame::new(Message::WriteClose { stream: STREAM }))?;
    Ok(0)
}

/// One file-protocol write: open a stream, wait for the client's ready ack, send
/// the data, close the stream. This is the handshake real tmux runs and the
/// conformance client plays the other side of.
fn write_stream<R, W>(
    reader: &mut R,
    writer: &mut W,
    stream: i32,
    fd: i32,
    data: &[u8],
) -> io::Result<()>
where
    R: FrameReader,
    W: FrameWriter,
{
    writer.send(Frame::new(Message::WriteOpen {
        stream,
        fd,
        flags: 0,
        path: Vec::new(),
    }))?;

    // Block until the client acknowledges this stream is open.
    loop {
        let frame = reader.recv()?;
        if frame.version != PROTOCOL_VERSION {
            writer.send(Frame::new(Message::Version))?;
            return Err(io::Error::other("client sent bad version mid-stream"));
        }
        match frame.msg {
            Message::WriteReady { stream: s, error } if s == stream => {
                if error == 0 {
                    // A tmux imsg is capped at 16 KiB including headers. Large
                    // captures must be streamed across multiple MSG_WRITE
                    // frames or the encoder rejects the oversized frame and
                    // leaves the client waiting forever for MSG_EXIT.
                    for chunk in data.chunks(8 * 1024) {
                        writer.send(Frame::new(Message::Write {
                            stream,
                            data: chunk.to_vec(),
                        }))?;
                    }
                }
                break;
            }
            // Ignore anything else (e.g. late identify/resize frames).
            _ => {}
        }
    }

    writer.send(Frame::new(Message::WriteClose { stream }))?;
    Ok(())
}

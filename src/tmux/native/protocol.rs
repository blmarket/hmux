//! Per-connection control-plane state machine for the native server.
//!
//! Speaks the client side of tmux's imsg protocol well enough for a *command*
//! client (e.g. `tmux list-sessions`): version check → identify → command
//! dispatch, delivering output over the imsg file protocol
//! (`MSG_WRITE_OPEN`/`MSG_WRITE_READY`/`MSG_WRITE`/`MSG_WRITE_CLOSE`) and ending
//! with `MSG_EXIT`. This is the exact exchange the conformance suite asserts on.
//!
//! The interactive *attach* path (identify carries a tty fd; the server would
//! then composite panes onto it) is now implemented in `super::attach`: on
//! `attach-session` the handler takes the client's tty fd and drives it
//! directly via libghostty-vt, matching the real tmux flow where terminal I/O
//! bypasses imsg after identify.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::integration::status::{PaneAgents, StatusHub};
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::traits::{FrameReader, FrameWriter};

use super::attach::{self, ClientTty};
use super::command;
use super::command::queue::{CommandQueue, QueueCompletion, QueueState, QueueTicket};
use super::pane::{NativePaneObservation, OutputSubscription};
use super::registry::{self, Resolution};
use super::state::{ClientAction, ControlStateSnapshot, ServerState};
use super::status;

/// stdout / stderr fds the client maps opened streams onto.
const FD_STDOUT: i32 = 1;
const FD_STDERR: i32 = 2;

/// How long a `StatusWait` long-poll parks before returning an unchanged
/// heartbeat. Doubles as dead-client detection: the next write fails if the peer
/// is gone. See PROTOCOL.md §semantics.
const STATUS_HEARTBEAT: Duration = Duration::from_secs(30);

const CONTROL_BUFFER_HIGH: usize = 8192;
const CLIENT_READONLY: i64 = 0x800;
const CLIENT_CONTROL: i64 = 0x2000;
const CLIENT_CONTROLCONTROL: i64 = 0x4000;
const CLIENT_UTF8: i64 = 0x10000;
const CLIENT_IGNORESIZE: i64 = 0x20000;
const CLIENT_CONTROL_NOOUTPUT: i64 = 0x4000000;
const CLIENT_ACTIVEPANE: i64 = 0x80000000;
const CLIENT_CONTROL_PAUSEAFTER: i64 = 0x100000000;
const CLIENT_CONTROL_WAITEXIT: i64 = 0x200000000;
const CLIENT_NO_DETACH_ON_DESTROY: i64 = 0x8000000000;

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
                        return attach::handle_attach(
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
                        return attach::handle_new_session(
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

            // hmux-private long-poll: park until the status hub advances past
            // `since` (or the heartbeat fires), then reply with a snapshot and
            // loop back to `recv` for the client's next `StatusWait`. Unlike the
            // one-shot command path, this connection stays open (PROTOCOL.md).
            Message::StatusWait { since } => {
                let snap = hub.wait_after(since, STATUS_HEARTBEAT);
                let body = {
                    let st = state
                        .lock()
                        .map_err(|_| io::Error::other("native server state poisoned"))?;
                    command::encode_status_body(&st, &snap.panes)
                };
                writer.send(Frame::new(Message::Status {
                    revision: snap.revision,
                    body,
                }))?;
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
    let session = match command::classify(args) {
        command::Intent::NewAttach => {
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?;
            command::new_session_for_attach(args, &mut st, context).map_err(io::Error::other)?
        }
        command::Intent::Attach => {
            let supplied_target = args
                .iter()
                .position(|arg| arg == "-t")
                .and_then(|index| args.get(index + 1))
                .map(String::as_str)
                .or_else(|| {
                    args.iter()
                        .find_map(|arg| arg.strip_prefix("-t").filter(|value| !value.is_empty()))
                })
                .map(|target| target.split(':').next().unwrap_or(target).to_string());
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?;
            let target = attach::attach_target(supplied_target, &mut st, context)
                .map_err(io::Error::other)?;
            if st.find(&target).is_none() {
                return Err(io::Error::other(format!("can't find session: {target}")));
            }
            target
        }
        command::Intent::Command => "0".to_string(),
    };

    // Control-mode clients are attached clients too. Keep registrations alive
    // for the lifetime of this loop so session/group attachment formats and
    // render invalidation see them just like interactive tty clients.
    let (render_registry, mut session_id) = {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("native server state poisoned"))?;
        let session_id = st
            .session_id(&session)
            .ok_or_else(|| io::Error::other(format!("can't find session: {session}")))?;
        (st.client_render_registry(), session_id)
    };
    let client_name = client_tty
        .tty_name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("client-{}", client_tty.client_pid.unwrap_or_default()));
    let mut options = control_client_options(args);
    let client_flags = options.client_flags(client_tty.flags);
    writer.send(Frame::new(Message::Flags(client_flags)))?;
    let render_attachment = render_registry.attach_with_details(
        session_id,
        client_name.clone(),
        client_tty.term.clone().unwrap_or_default(),
        client_tty.client_pid,
        80,
        24,
        options.display_flags(client_tty.flags),
        options.read_only,
        true,
    )?;

    let stdin = client_tty
        .stdin
        .as_ref()
        .ok_or_else(|| io::Error::other("control client has no stdin"))?
        .as_raw_fd();
    let control_control_mode = client_tty.flags & CLIENT_CONTROLCONTROL != 0;
    let output = if control_control_mode {
        stdin
    } else {
        client_tty
            .stdout
            .as_ref()
            .ok_or_else(|| io::Error::other("control client has no stdout"))?
            .as_raw_fd()
    };
    let mut control_writer = ControlWriter::new(output)?;
    let mut subscriptions = ControlSubscriptions::default();
    let mut format_cache = status::RenderCache::for_client(status::ClientContext {
        term: client_tty.term.clone(),
        tty: client_tty.tty_name.clone(),
        pid: client_tty.client_pid,
        cwd: context.cwd.clone(),
        environment: context.environment.clone(),
        control_mode: true,
        read_only: options.read_only,
        flags: options.display_flags(client_tty.flags),
    });
    let mut stable_session = format!("${session_id}");
    let (mut snapshot, mut checkpoint) = {
        let state = state
            .lock()
            .map_err(|_| io::Error::other("native server state poisoned"))?;
        (
            state
                .control_snapshot(&stable_session)
                .ok_or_else(|| io::Error::other(format!("can't find session: {session}")))?,
            state.control_checkpoint_end(),
        )
    };
    let mut streams = control_pane_streams(&snapshot)?;
    let mut sequence = 0u64;
    if control_control_mode {
        control_writer.enqueue(b"\x1bP1000p");
    }
    let initial = ControlCommandId::next(&mut sequence);
    write_control_marker(&mut control_writer, "%begin", initial, 0);
    write_control_marker(&mut control_writer, "%end", initial, 0);
    control_writer.enqueue_line(format!(
        "%session-changed ${} {}",
        snapshot.session_id, snapshot.session_name
    ));
    {
        let errors = state
            .lock()
            .map_err(|_| io::Error::other("native server state poisoned"))?
            .take_config_errors();
        for error in errors {
            control_writer.enqueue_line(format!("%config-error {error}"));
        }
    }
    control_writer.flush()?;

    let mut pending = Vec::new();
    let mut buffer = [0u8; 4096];
    let mut client_size = None;
    let mut client_window_sizes = BTreeMap::new();
    let mut command_queue = CommandQueue::new();
    let mut pending_control_command: Option<PendingControlCommand> = None;
    let mut stdin_open = true;
    let mut exit_status = 0;
    let mut injected_prefix_pending = false;
    let mut control_context = context.clone();
    control_context.tty_name = Some(client_name.clone());
    control_context.current_session_id = Some(session_id);
    control_context.read_only = options.read_only;
    control_context.preserve_queue_insertions = true;
    control_context.active_panes = options
        .active_pane
        .then(|| Arc::new(Mutex::new(BTreeMap::new())));
    loop {
        for message in render_attachment.take_messages() {
            control_writer.enqueue_line(format!("%message {}", message.text));
        }
        let requested_switch = match render_attachment.take_action() {
            Some(ClientAction::Switch(session_id)) => Some(session_id),
            Some(ClientAction::Detach) => {
                control_writer.drain()?;
                writer.send(Frame::new(Message::Exit(Some(exit_status))))?;
                return Ok(());
            }
            Some(ClientAction::Lock(command)) => {
                writer.send(Frame::new(Message::Lock(command)))?;
                None
            }
            // Control clients have no terminal job to stop.
            Some(ClientAction::Suspend) => None,
            // Control clients have no tty on which to emit a selection sequence.
            Some(ClientAction::SetSelection(_)) => None,
            Some(ClientAction::Keys(keys)) => {
                if attach::dispatch_control_client_keys(
                    &keys,
                    &mut injected_prefix_pending,
                    state,
                    &stable_session,
                    hub,
                    &control_context,
                ) {
                    control_writer.drain()?;
                    writer.send(Frame::new(Message::Exit(Some(exit_status))))?;
                    return Ok(());
                }
                None
            }
            // Control clients have no tty compositor on which to draw an
            // overlay. The command is accepted, matching tmux's no-render path.
            Some(ClientAction::Overlay { reply, .. } | ClientAction::Confirm { reply, .. }) => {
                if let Some(reply) = reply {
                    let _ = reply.send(super::state::PromptCompletion {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit: 0,
                        inserted: false,
                    });
                }
                None
            }
            None => None,
        };
        let replacement = {
            let state = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?;
            if let Some(next_session_id) = requested_switch {
                let stable = format!("${next_session_id}");
                state.control_snapshot(&stable).map(|_| {
                    (
                        next_session_id,
                        stable,
                        state.control_checkpoint_end(),
                        false,
                    )
                })
            } else if state.control_snapshot(&stable_session).is_some() {
                None
            } else if options.no_detach_on_destroy {
                state.sessions().last().map(|session| {
                    let stable = format!("${}", session.id);
                    (session.id, stable, state.control_checkpoint_end(), true)
                })
            } else {
                control_writer.enqueue_line("%sessions-changed");
                control_writer.drain()?;
                writer.send(Frame::new(Message::Exit(Some(exit_status))))?;
                return Ok(());
            }
        };
        if let Some((next_session_id, next_stable, next_checkpoint, session_destroyed)) =
            replacement
        {
            session_id = next_session_id;
            stable_session = next_stable;
            checkpoint = next_checkpoint;
            render_attachment.update_session(session_id);
            control_context.current_session_id = Some(session_id);
            if let Some(active_panes) = &control_context.active_panes {
                if let Ok(mut active_panes) = active_panes.lock() {
                    active_panes.clear();
                }
            }
            let next_snapshot = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?
                .control_snapshot(&stable_session)
                .ok_or_else(|| io::Error::other("fallback control session disappeared"))?;
            control_writer.enqueue_line(format!(
                "%session-changed ${} {}",
                next_snapshot.session_id, next_snapshot.session_name
            ));
            if session_destroyed {
                control_writer.enqueue_line("%sessions-changed");
                for window_id in snapshot
                    .global_windows
                    .keys()
                    .filter(|window_id| !next_snapshot.global_windows.contains_key(window_id))
                {
                    control_writer.enqueue_line(format!("%unlinked-window-close @{window_id}"));
                }
            }
            streams = control_pane_streams(&next_snapshot)?;
            snapshot = next_snapshot;
        }
        let command_pending = pending_control_command.is_some();
        let pane_ids = streams.keys().copied().collect::<Vec<_>>();
        let mut pollfds = Vec::with_capacity(4 + pane_ids.len());
        pollfds.push(libc::pollfd {
            fd: if stdin_open { stdin } else { -1 },
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        });
        pollfds.push(libc::pollfd {
            fd: if command_pending {
                -1
            } else {
                render_attachment.as_raw_fd()
            },
            events: libc::POLLIN,
            revents: 0,
        });
        pollfds.push(libc::pollfd {
            fd: output,
            events: if control_writer.has_pending() {
                libc::POLLOUT
            } else {
                0
            },
            revents: 0,
        });
        pollfds.push(libc::pollfd {
            fd: pending_control_command
                .as_ref()
                .map_or(-1, PendingControlCommand::as_raw_fd),
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        });
        pollfds.extend(pane_ids.iter().filter_map(|pane_id| {
            streams.get(pane_id).map(|stream| libc::pollfd {
                fd: if command_pending {
                    -1
                } else {
                    stream.subscription.as_raw_fd()
                },
                events: libc::POLLIN | libc::POLLHUP,
                revents: 0,
            })
        }));
        let now = Instant::now();
        let subscription_timeout = if command_pending {
            -1
        } else {
            subscriptions.poll_timeout(now)
        };
        let alert_timeout = if command_pending {
            -1
        } else {
            state
                .lock()
                .ok()
                .and_then(|state| state.alert_poll_timeout())
                .map(|duration| duration.as_millis().min(i32::MAX as u128) as i32)
                .unwrap_or(-1)
        };
        let timeout = match (subscription_timeout, alert_timeout) {
            (-1, timeout) | (timeout, -1) => timeout,
            (left, right) => left.min(right),
        };
        let timeout = if pending_control_command.is_none()
            && matches!(command_queue.state(), QueueState::Ready)
        {
            0
        } else {
            timeout
        };
        let ready = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as _, timeout) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        let stdin_ready = pollfds[0].revents & (libc::POLLIN | libc::POLLHUP) != 0;
        let state_ready = pollfds[1].revents & libc::POLLIN != 0;
        let output_ready = pollfds[2].revents & libc::POLLOUT != 0;
        let command_ready = pollfds[3].revents & (libc::POLLIN | libc::POLLHUP) != 0;
        let ready_panes = pane_ids
            .iter()
            .zip(pollfds.iter().skip(4))
            .filter_map(|(pane_id, pollfd)| {
                (pollfd.revents & (libc::POLLIN | libc::POLLHUP) != 0).then_some(*pane_id)
            })
            .collect::<Vec<_>>();

        if output_ready {
            control_writer.flush()?;
        }

        if command_ready {
            let pending = pending_control_command
                .take()
                .ok_or_else(|| io::Error::other("control command readiness without command"))?;
            let (ticket, id, mut result) = pending.take_result()?;
            if !result.stdout_data().is_empty() {
                control_writer.enqueue(result.stdout_data());
            }
            if !result.stderr.is_empty() {
                control_writer.enqueue(result.stderr.as_bytes());
            }
            if result.exit == 0 {
                write_control_marker(&mut control_writer, "%end", id, 1);
            } else {
                exit_status = 1;
                write_control_marker(&mut control_writer, "%error", id, 1);
            }
            let errors = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?
                .take_config_errors();
            for error in errors {
                control_writer.enqueue_line(format!("%config-error {error}"));
            }
            advance_control_snapshot(
                state,
                session_id,
                &stable_session,
                &mut checkpoint,
                &mut snapshot,
                &mut streams,
                &mut control_writer,
            )?;
            write_control_output(&mut control_writer, &mut streams, &options)?;
            let discard_group_tail = result.exit != 0 && !result.continue_queue;
            let inserted = std::mem::take(&mut result.inserted_results);
            let insert_next = inserted
                .into_iter()
                .map(|result| vec![ControlQueueItem::Completed(result)])
                .collect();
            command_queue
                .resume(ticket)
                .map_err(|_| io::Error::other("stale control command resume"))?;
            command_queue
                .complete(
                    ticket,
                    QueueCompletion {
                        discard_group_tail,
                        insert_next,
                    },
                )
                .map_err(|_| io::Error::other("stale control command completion"))?;
        }

        if stdin_ready {
            let count = unsafe { libc::read(stdin, buffer.as_mut_ptr().cast(), buffer.len()) };
            if count < 0 {
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::WouldBlock {
                    return Err(error);
                }
            } else if count == 0 {
                stdin_open = false;
            } else {
                pending.extend_from_slice(&buffer[..count as usize]);
            }
        }

        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            let raw_line = String::from_utf8_lossy(&pending[..newline]).to_string();
            pending.drain(..=newline);
            if raw_line.is_empty() {
                stdin_open = false;
                pending.clear();
                break;
            }
            let line = raw_line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            let aliases = state
                .lock()
                .map_err(|_| io::Error::other("native server state poisoned"))?
                .command_aliases();
            let groups = match command::command_string_groups_with_aliases(&line, &aliases) {
                Ok(groups) => groups,
                Err(result) => {
                    command_queue.push_back_group([ControlQueueItem::ParseError(result)]);
                    continue;
                }
            };
            if !groups.is_empty() {
                command_queue.push_back_group(
                    groups
                        .into_iter()
                        .map(ControlQueueItem::Command)
                        .collect::<Vec<_>>(),
                );
            }
        }

        while pending_control_command.is_none() {
            let Some(started) = command_queue.start_next() else {
                break;
            };
            let ticket = started.ticket;
            let argv = match started.value {
                ControlQueueItem::Completed(result) => {
                    let id = ControlCommandId::next(&mut sequence);
                    let flags = result.control_flags;
                    write_control_marker(&mut control_writer, "%begin", id, flags);
                    if !result.stdout_data().is_empty() {
                        control_writer.enqueue(result.stdout_data());
                    }
                    if !result.stderr.is_empty() {
                        control_writer.enqueue(result.stderr.as_bytes());
                    }
                    if result.exit == 0 {
                        write_control_marker(&mut control_writer, "%end", id, flags);
                    } else {
                        exit_status = result.exit;
                        write_control_marker(&mut control_writer, "%error", id, flags);
                    }
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale completed control queue item"))?;
                    continue;
                }
                ControlQueueItem::ParseError(result) => {
                    let id = ControlCommandId::next(&mut sequence);
                    write_control_marker(&mut control_writer, "%begin", id, 1);
                    control_writer.enqueue(b"parse error: ");
                    control_writer.enqueue(result.stderr.as_bytes());
                    write_control_marker(&mut control_writer, "%error", id, 1);
                    command_queue
                        .complete(ticket, QueueCompletion::failed())
                        .map_err(|_| io::Error::other("stale control parse error"))?;
                    continue;
                }
                ControlQueueItem::Command(argv) => argv,
            };

            {
                let id = ControlCommandId::next(&mut sequence);
                write_control_marker(&mut control_writer, "%begin", id, 1);
                let refresh_flags = control_refresh_flag_values(&argv);
                let reset_output_offsets = refresh_flags.iter().any(|value| {
                    value
                        .split(',')
                        .any(|flag| flag.strip_prefix('!').unwrap_or(flag) == "no-output")
                });
                for value in &refresh_flags {
                    options.apply_flags(value);
                }
                let switch_read_only = argv.first().is_some_and(|name| {
                    matches!(registry::resolve(name), Resolution::Name("switch-client"))
                }) && control_command_has_flag(&argv, 'r');
                if switch_read_only {
                    let enable = !options.read_only;
                    options.read_only = enable;
                    options.ignore_size = enable;
                }
                let flags_changed = !refresh_flags.is_empty() || switch_read_only;
                if flags_changed {
                    let display_flags = options.display_flags(client_tty.flags);
                    render_attachment
                        .update_control_flags(display_flags.clone(), options.read_only);
                    format_cache.update_client_flags(display_flags, options.read_only);
                    control_context.read_only = options.read_only;
                    if options.active_pane && control_context.active_panes.is_none() {
                        control_context.active_panes = Some(Arc::new(Mutex::new(BTreeMap::new())));
                    } else if !options.active_pane {
                        control_context.active_panes = None;
                    }
                    if reset_output_offsets {
                        for stream in streams.values_mut() {
                            stream.offset = stream.observation.control_output_end();
                            stream.pending_since = None;
                        }
                    }
                    writer.send(Frame::new(Message::Flags(
                        options.client_flags(client_tty.flags),
                    )))?;
                }
                let handled_colour_report = apply_control_colour_report(&argv, state)?;
                let handled_offsets =
                    apply_control_offset_actions(&argv, &mut streams, &mut control_writer);
                let handled_subscriptions =
                    apply_control_subscription_actions(&argv, &mut subscriptions);
                if handled_offsets || handled_subscriptions {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                if let Some(size) = control_size_action(&argv) {
                    let mut st = state
                        .lock()
                        .map_err(|_| io::Error::other("native server state poisoned"))?;
                    match size {
                        ControlSizeAction::Client(cols, rows) => {
                            client_size = Some((cols, rows));
                            render_attachment.update_size(cols, rows);
                            if !control_size_is_ignored(&st, session_id, &client_name, &options) {
                                let _ = st.resize_session(&stable_session, cols, rows);
                                for (window_id, (cols, rows)) in &client_window_sizes {
                                    let _ = st.resize_linked_window(
                                        &format!("@{window_id}"),
                                        *cols,
                                        *rows,
                                    );
                                }
                            }
                        }
                        ControlSizeAction::Window(window_id, Some((cols, rows))) => {
                            client_window_sizes.insert(window_id, (cols, rows));
                            render_attachment.mark_size_changed();
                            if !control_size_is_ignored(&st, session_id, &client_name, &options) {
                                let _ =
                                    st.resize_linked_window(&format!("@{window_id}"), cols, rows);
                            }
                        }
                        ControlSizeAction::Window(window_id, None) => {
                            client_window_sizes.remove(&window_id);
                            let (cols, rows) = client_size.unwrap_or((80, 24));
                            render_attachment.mark_size_changed();
                            if !control_size_is_ignored(&st, session_id, &client_name, &options) {
                                let _ =
                                    st.resize_linked_window(&format!("@{window_id}"), cols, rows);
                            }
                        }
                    }
                    let next = st.control_snapshot(&stable_session);
                    drop(st);
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    if let Some(next) = next {
                        write_all_control_layouts(&mut control_writer, &next);
                        sync_control_pane_streams(&next, &mut streams)?;
                        snapshot = next;
                    }
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                if is_control_refresh_operation(&argv) {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                if handled_colour_report {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                if switch_read_only {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    control_writer.enqueue_line(format!(
                        "%session-changed ${} {}",
                        snapshot.session_id, snapshot.session_name
                    ));
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                if !refresh_flags.is_empty() {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                    command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale control queue item"))?;
                    continue;
                }
                let agents = hub.snapshot().panes;
                if control_command_may_block(&argv) {
                    command_queue
                        .wait(ticket)
                        .map_err(|_| io::Error::other("stale control queue wait"))?;
                    pending_control_command = Some(PendingControlCommand::start(
                        ticket,
                        id,
                        argv,
                        Arc::clone(state),
                        agents,
                        control_context.clone(),
                    )?);
                    break;
                }
                let switches_client = argv.first().is_some_and(|name| {
                    matches!(registry::resolve(name), Resolution::Name("switch-client"))
                });
                let mut result = command::run_with_context(&argv, state, &agents, &control_context);
                if result.exit == 0
                    && matches!(
                        argv.first().map(String::as_str),
                        Some("select-window" | "selectw")
                    )
                {
                    if let (Some((cols, rows)), Some(target)) = (
                        client_size,
                        argv.iter()
                            .position(|word| word == "-t")
                            .and_then(|index| argv.get(index + 1)),
                    ) {
                        let _ = state
                            .lock()
                            .map_err(|_| io::Error::other("native server state poisoned"))?
                            .resize_linked_window(target, cols, rows);
                    }
                }
                if !result.stdout_data().is_empty() {
                    control_writer.enqueue(result.stdout_data());
                }
                if !result.stderr.is_empty() {
                    control_writer.enqueue(result.stderr.as_bytes());
                }
                if result.exit == 0 {
                    write_control_marker(&mut control_writer, "%end", id, 1);
                } else {
                    exit_status = 1;
                    write_control_marker(&mut control_writer, "%error", id, 1);
                }
                let errors = state
                    .lock()
                    .map_err(|_| io::Error::other("native server state poisoned"))?
                    .take_config_errors();
                for error in errors {
                    control_writer.enqueue_line(format!("%config-error {error}"));
                }
                if result.exit != 0 || !switches_client {
                    advance_control_snapshot(
                        state,
                        session_id,
                        &stable_session,
                        &mut checkpoint,
                        &mut snapshot,
                        &mut streams,
                        &mut control_writer,
                    )?;
                    write_control_output(&mut control_writer, &mut streams, &options)?;
                }
                let discard_group_tail = result.exit != 0 && !result.continue_queue;
                let inserted = std::mem::take(&mut result.inserted_results);
                let insert_next = inserted
                    .into_iter()
                    .map(|result| vec![ControlQueueItem::Completed(result)])
                    .collect();
                command_queue
                    .complete(
                        ticket,
                        QueueCompletion {
                            discard_group_tail,
                            insert_next,
                        },
                    )
                    .map_err(|_| io::Error::other("stale control command result"))?;
            }
        }

        if pending_control_command.is_none() {
            if state_ready {
                let _ = render_attachment.take();
                advance_control_snapshot(
                    state,
                    session_id,
                    &stable_session,
                    &mut checkpoint,
                    &mut snapshot,
                    &mut streams,
                    &mut control_writer,
                )?;
            }
            let pane_state_ready = !ready_panes.is_empty();
            for pane_id in ready_panes {
                if let Some(stream) = streams.get_mut(&pane_id) {
                    stream.subscription.drain();
                    stream.pending_since.get_or_insert_with(Instant::now);
                }
            }
            let alert_changed = {
                let mut state = state
                    .lock()
                    .map_err(|_| io::Error::other("native server state poisoned"))?;
                let changed = state.refresh_alerts(Instant::now());
                if changed {
                    state.record_control_checkpoint();
                }
                changed
            };
            if alert_changed {
                advance_control_snapshot(
                    state,
                    session_id,
                    &stable_session,
                    &mut checkpoint,
                    &mut snapshot,
                    &mut streams,
                    &mut control_writer,
                )?;
            }
            write_control_output(&mut control_writer, &mut streams, &options)?;
            if pane_state_ready {
                {
                    let mut state = state
                        .lock()
                        .map_err(|_| io::Error::other("native server state poisoned"))?;
                    state.reap_exited_panes();
                    state.record_control_checkpoint();
                }
                advance_control_snapshot(
                    state,
                    session_id,
                    &stable_session,
                    &mut checkpoint,
                    &mut snapshot,
                    &mut streams,
                    &mut control_writer,
                )?;
            }
            if subscriptions
                .next_check
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                let (cols, rows) = client_size.unwrap_or((80, 24));
                let state = state
                    .lock()
                    .map_err(|_| io::Error::other("native server state poisoned"))?;
                check_control_subscriptions(
                    &mut subscriptions,
                    &mut format_cache,
                    &state,
                    &stable_session,
                    cols,
                    rows,
                    &mut control_writer,
                );
                subscriptions.reschedule(Instant::now());
            }
        }
        control_writer.flush()?;
        if !stdin_open
            && pending_control_command.is_none()
            && matches!(command_queue.state(), QueueState::Empty)
        {
            write_control_output(&mut control_writer, &mut streams, &options)?;
            control_writer.drain()?;
            writer.send(Frame::new(Message::Exit(Some(exit_status))))?;
            return Ok(());
        }
    }
}

struct ControlWriter {
    fd: i32,
    blocks: VecDeque<Vec<u8>>,
    front_offset: usize,
    queued: usize,
}

impl ControlWriter {
    fn new(fd: i32) -> io::Result<Self> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            fd,
            blocks: VecDeque::new(),
            front_offset: 0,
            queued: 0,
        })
    }

    fn enqueue(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.queued = self.queued.saturating_add(bytes.len());
        self.blocks.push_back(bytes.to_vec());
    }

    fn enqueue_line(&mut self, line: impl AsRef<[u8]>) {
        self.enqueue(line.as_ref());
        self.enqueue(b"\n");
    }

    fn available(&self) -> usize {
        CONTROL_BUFFER_HIGH.saturating_sub(self.queued)
    }

    fn has_pending(&self) -> bool {
        !self.blocks.is_empty()
    }

    fn flush(&mut self) -> io::Result<()> {
        while let Some(front) = self.blocks.front() {
            let bytes = &front[self.front_offset..];
            let written = unsafe { libc::write(self.fd, bytes.as_ptr().cast(), bytes.len()) };
            if written < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                return Err(error);
            }
            if written == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "control client output closed",
                ));
            }
            let written = written as usize;
            self.front_offset += written;
            self.queued = self.queued.saturating_sub(written);
            if self.front_offset == front.len() {
                self.blocks.pop_front();
                self.front_offset = 0;
            }
        }
        Ok(())
    }

    fn drain(&mut self) -> io::Result<()> {
        while self.has_pending() {
            self.flush()?;
            if !self.has_pending() {
                break;
            }
            let mut pollfd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLOUT,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut pollfd, 1, -1) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
        }
        Ok(())
    }
}

enum ControlQueueItem {
    Command(Vec<String>),
    ParseError(command::CommandResult),
    Completed(command::CommandResult),
}

struct PendingControlCommand {
    ticket: QueueTicket,
    id: ControlCommandId,
    completion: UnixStream,
    result: Arc<Mutex<Option<command::CommandResult>>>,
}

impl PendingControlCommand {
    fn start(
        ticket: QueueTicket,
        id: ControlCommandId,
        args: Vec<String>,
        state: Arc<Mutex<ServerState>>,
        agents: PaneAgents,
        context: command::ClientContext,
    ) -> io::Result<Self> {
        let (completion, mut signal) = UnixStream::pair()?;
        completion.set_nonblocking(true)?;
        let result = Arc::new(Mutex::new(None));
        let worker_result = Arc::clone(&result);
        thread::spawn(move || {
            let completed = command::run_with_context(&args, &state, &agents, &context);
            if let Ok(mut result) = worker_result.lock() {
                *result = Some(completed);
            }
            let _ = signal.write_all(&[1]);
        });
        Ok(Self {
            ticket,
            id,
            completion,
            result,
        })
    }

    fn as_raw_fd(&self) -> i32 {
        self.completion.as_raw_fd()
    }

    fn take_result(
        mut self,
    ) -> io::Result<(QueueTicket, ControlCommandId, command::CommandResult)> {
        let mut byte = [0u8; 1];
        let _ = self.completion.read(&mut byte);
        let result = self
            .result
            .lock()
            .map_err(|_| io::Error::other("control command result poisoned"))?
            .take()
            .ok_or_else(|| io::Error::other("control command completed without a result"))?;
        Ok((self.ticket, self.id, result))
    }
}

fn control_command_may_block(args: &[String]) -> bool {
    let Some(Resolution::Name(name)) = args.first().map(|name| registry::resolve(name)) else {
        return false;
    };
    match name {
        "wait-for" => !['L', 'S', 'U']
            .into_iter()
            .any(|flag| control_command_has_flag(args, flag)),
        "run-shell" | "if-shell" => !control_command_has_flag(args, 'b'),
        "command-prompt" => {
            !control_command_has_flag(args, 'b') && !control_command_has_flag(args, 'i')
        }
        "confirm-before" | "display-panes" => !control_command_has_flag(args, 'b'),
        "display-menu" => true,
        "display-popup" => !control_command_has_flag(args, 'C'),
        "source-file" | "load-buffer" | "save-buffer" => true,
        _ => false,
    }
}

fn control_command_has_flag(args: &[String], flag: char) -> bool {
    args.iter()
        .skip(1)
        .take_while(|argument| argument.as_str() != "--")
        .filter_map(|argument| argument.strip_prefix('-'))
        .filter(|flags| !flags.starts_with('-'))
        .any(|flags| flags.contains(flag))
}

#[derive(Clone, Copy)]
struct ControlCommandId {
    time: u64,
    number: u64,
}

impl ControlCommandId {
    fn next(sequence: &mut u64) -> Self {
        let id = Self {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            number: *sequence,
        };
        *sequence = sequence.saturating_add(1);
        id
    }
}

struct ControlPaneStream {
    runtime_id: u64,
    offset: u64,
    observation: Arc<NativePaneObservation>,
    subscription: OutputSubscription,
    enabled: bool,
    paused: bool,
    pending_since: Option<Instant>,
}

#[derive(Default)]
struct ControlClientOptions {
    pause_after: Option<Duration>,
    no_output: bool,
    wait_exit: bool,
    read_only: bool,
    ignore_size: bool,
    active_pane: bool,
    no_detach_on_destroy: bool,
}

impl ControlClientOptions {
    fn apply_flags(&mut self, value: &str) {
        for flag in value.split(',') {
            let (clear, flag) = flag
                .strip_prefix('!')
                .map_or((false, flag), |flag| (true, flag));
            if flag == "pause-after" || flag.starts_with("pause-after=") {
                if clear {
                    self.pause_after = None;
                } else {
                    let seconds = flag
                        .strip_prefix("pause-after=")
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    self.pause_after = Some(Duration::from_secs(seconds));
                }
            } else {
                let enabled = !clear;
                match flag {
                    "no-output" => self.no_output = enabled,
                    "wait-exit" => self.wait_exit = enabled,
                    // An established read-only client cannot clear itself with
                    // refresh-client; switch-client -r is the owner escape.
                    "read-only" if enabled || !self.read_only => self.read_only = enabled,
                    "ignore-size" => self.ignore_size = enabled,
                    "active-pane" => self.active_pane = enabled,
                    "no-detach-on-destroy" => self.no_detach_on_destroy = enabled,
                    _ => {}
                }
            }
        }
    }

    fn client_flags(&self, identified: i64) -> i64 {
        let mut flags = identified;
        for (enabled, flag) in [
            (self.no_output, CLIENT_CONTROL_NOOUTPUT),
            (self.wait_exit, CLIENT_CONTROL_WAITEXIT),
            (self.pause_after.is_some(), CLIENT_CONTROL_PAUSEAFTER),
            (self.read_only, CLIENT_READONLY),
            (self.ignore_size, CLIENT_IGNORESIZE),
            (self.active_pane, CLIENT_ACTIVEPANE),
            (self.no_detach_on_destroy, CLIENT_NO_DETACH_ON_DESTROY),
        ] {
            if enabled {
                flags |= flag;
            } else {
                flags &= !flag;
            }
        }
        flags
    }

    fn display_flags(&self, identified: i64) -> String {
        let mut flags = vec!["attached", "focused", "control-mode"];
        if self.ignore_size {
            flags.push("ignore-size");
        }
        if self.no_detach_on_destroy {
            flags.push("no-detach-on-destroy");
        }
        if self.no_output {
            flags.push("no-output");
        }
        if self.wait_exit {
            flags.push("wait-exit");
        }
        let pause_after = self
            .pause_after
            .map(|duration| format!("pause-after={}", duration.as_secs()));
        if let Some(pause_after) = pause_after.as_deref() {
            flags.push(pause_after);
        }
        if self.read_only {
            flags.push("read-only");
        }
        if self.active_pane {
            flags.push("active-pane");
        }
        if identified & CLIENT_UTF8 != 0 {
            flags.push("UTF-8");
        }
        flags.join(",")
    }
}

#[derive(Clone, Copy)]
enum ControlSubscriptionKind {
    Session,
    Pane(u32),
    AllPanes,
    Window(u32),
    AllWindows,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ControlSubscriptionTarget {
    window_id: Option<u32>,
    window_index: Option<u32>,
    pane_id: Option<u32>,
}

struct ControlSubscription {
    kind: ControlSubscriptionKind,
    format: String,
    last: BTreeMap<ControlSubscriptionTarget, String>,
}

#[derive(Default)]
struct ControlSubscriptions {
    entries: BTreeMap<String, ControlSubscription>,
    next_check: Option<Instant>,
}

impl ControlSubscriptions {
    fn poll_timeout(&self, now: Instant) -> i32 {
        self.next_check.map_or(-1, |deadline| {
            deadline
                .saturating_duration_since(now)
                .as_millis()
                .min(i32::MAX as u128) as i32
        })
    }

    fn reschedule(&mut self, now: Instant) {
        self.next_check = (!self.entries.is_empty()).then(|| now + Duration::from_secs(1));
    }
}

fn control_client_options(args: &[String]) -> ControlClientOptions {
    let mut options = ControlClientOptions::default();
    let mut index = 0;
    while index < args.len() {
        let value = if args[index] == "-f" {
            index += 1;
            args.get(index).map(String::as_str)
        } else {
            args[index]
                .strip_prefix("-f")
                .filter(|value| !value.is_empty())
        };
        if let Some(value) = value {
            options.apply_flags(value);
        }
        index += 1;
    }
    if attach_command_has_flag(args, 'r') {
        options.read_only = true;
        options.ignore_size = true;
    }
    options
}

fn attach_command_has_flag(args: &[String], wanted: char) -> bool {
    let mut arguments = args.iter().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            break;
        }
        let Some(flags) = argument.strip_prefix('-') else {
            continue;
        };
        let mut flags = flags.chars().peekable();
        while let Some(flag) = flags.next() {
            if flag == wanted {
                return true;
            }
            if matches!(flag, 'c' | 'f' | 't') {
                if flags.peek().is_none() {
                    let _ = arguments.next();
                }
                break;
            }
        }
    }
    false
}

fn control_refresh_flag_values(args: &[String]) -> Vec<&str> {
    let Some(Resolution::Name("refresh-client")) = args.first().map(|name| registry::resolve(name))
    else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut index = 1;
    while index < args.len() {
        let argument = &args[index];
        if matches!(argument.as_str(), "-f" | "-F") {
            index += 1;
            if let Some(value) = args.get(index) {
                values.push(value.as_str());
            }
        } else if let Some(value) = argument
            .strip_prefix("-f")
            .or_else(|| argument.strip_prefix("-F"))
            .filter(|value| !value.is_empty())
        {
            values.push(value);
        }
        index += 1;
    }
    values
}

fn apply_control_colour_report(
    args: &[String],
    state: &Arc<Mutex<ServerState>>,
) -> io::Result<bool> {
    let Some(Resolution::Name("refresh-client")) = args.first().map(|name| registry::resolve(name))
    else {
        return Ok(false);
    };
    let value = args
        .iter()
        .position(|argument| argument == "-r")
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
        .or_else(|| {
            args.iter().find_map(|argument| {
                argument
                    .strip_prefix("-r")
                    .filter(|value| !value.is_empty())
            })
        });
    let Some(value) = value else {
        return Ok(false);
    };
    let Some((pane, report)) = value.split_once(':') else {
        return Ok(true);
    };
    if parse_control_colour_report(report).is_none() {
        return Ok(true);
    }
    let _ = state
        .lock()
        .map_err(|_| io::Error::other("native server state poisoned"))?
        .report_pane_control_colour(pane, report.as_bytes());
    Ok(true)
}

fn parse_control_colour_report(report: &str) -> Option<(bool, String)> {
    let (foreground, value) = report
        .strip_prefix("\x1b]10;")
        .map(|value| (true, value))
        .or_else(|| report.strip_prefix("\x1b]11;").map(|value| (false, value)))?;
    let value = value
        .strip_suffix('\u{7}')
        .or_else(|| value.strip_suffix("\x1b\\"))?;
    let value = value.strip_prefix("rgb:")?;
    let mut components = value.split('/');
    let red = scale_x11_colour(components.next()?)?;
    let green = scale_x11_colour(components.next()?)?;
    let blue = scale_x11_colour(components.next()?)?;
    if components.next().is_some() {
        return None;
    }
    Some((foreground, format!("#{red:02x}{green:02x}{blue:02x}")))
}

fn scale_x11_colour(component: &str) -> Option<u8> {
    if component.is_empty() || component.len() > 4 {
        return None;
    }
    let value = u32::from_str_radix(component, 16).ok()?;
    let maximum = (1u32 << (component.len() * 4)) - 1;
    Some(((value * 255 + maximum / 2) / maximum) as u8)
}

fn control_pane_streams(
    snapshot: &ControlStateSnapshot,
) -> io::Result<BTreeMap<u32, ControlPaneStream>> {
    let mut streams = BTreeMap::new();
    for pane in snapshot.windows.values().flat_map(|window| &window.panes) {
        let offset = pane.observation.control_output_end();
        let subscription = pane.observation.subscribe_output()?;
        streams.insert(
            pane.id,
            ControlPaneStream {
                runtime_id: pane.runtime_id,
                offset,
                observation: Arc::clone(&pane.observation),
                subscription,
                enabled: true,
                paused: false,
                pending_since: None,
            },
        );
    }
    Ok(streams)
}

fn sync_control_pane_streams(
    snapshot: &ControlStateSnapshot,
    streams: &mut BTreeMap<u32, ControlPaneStream>,
) -> io::Result<()> {
    let panes = snapshot
        .windows
        .values()
        .flat_map(|window| &window.panes)
        .map(|pane| (pane.id, pane))
        .collect::<BTreeMap<_, _>>();
    streams.retain(|pane_id, stream| {
        panes
            .get(pane_id)
            .is_some_and(|pane| pane.runtime_id == stream.runtime_id)
    });
    for (pane_id, pane) in panes {
        if streams.contains_key(&pane_id) {
            continue;
        }
        streams.insert(
            pane_id,
            ControlPaneStream {
                runtime_id: pane.runtime_id,
                offset: 0,
                observation: Arc::clone(&pane.observation),
                subscription: pane.observation.subscribe_output()?,
                enabled: true,
                paused: false,
                pending_since: Some(Instant::now()),
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn advance_control_snapshot(
    state: &Arc<Mutex<ServerState>>,
    session_id: u32,
    stable_session: &str,
    checkpoint: &mut u64,
    snapshot: &mut ControlStateSnapshot,
    streams: &mut BTreeMap<u32, ControlPaneStream>,
    writer: &mut ControlWriter,
) -> io::Result<()> {
    let (end, mut updates, current) = {
        let state = state
            .lock()
            .map_err(|_| io::Error::other("native server state poisoned"))?;
        let (end, updates) = state.control_checkpoints_since(session_id, *checkpoint);
        (end, updates, state.control_snapshot(stable_session))
    };
    *checkpoint = end;
    if let Some(current) = current {
        updates.push(current);
    }
    for next in updates {
        write_control_notifications(writer, snapshot, &next);
        sync_control_pane_streams(&next, streams)?;
        *snapshot = next;
    }
    Ok(())
}

fn write_control_marker(writer: &mut ControlWriter, marker: &str, id: ControlCommandId, flags: u8) {
    writer.enqueue_line(format!("{marker} {} {} {flags}", id.time, id.number));
}

fn write_control_output(
    writer: &mut ControlWriter,
    streams: &mut BTreeMap<u32, ControlPaneStream>,
    options: &ControlClientOptions,
) -> io::Result<()> {
    if options.no_output {
        for stream in streams.values_mut() {
            stream.offset = stream.observation.control_output_end();
            stream.pending_since = None;
        }
        return Ok(());
    }
    for (pane_id, stream) in streams {
        if !stream.enabled || stream.paused {
            continue;
        }
        let available = writer.available();
        if available <= 64 {
            break;
        }
        let raw_limit = (available - 64) / 4;
        let (next_offset, end, bytes) = stream
            .observation
            .control_output_chunk(stream.offset, raw_limit.max(1));
        if bytes.is_empty() {
            if next_offset == end {
                stream.pending_since = None;
            }
            continue;
        }
        let since = stream.pending_since.get_or_insert_with(Instant::now);
        let age = since.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        if options
            .pause_after
            .is_some_and(|pause_after| age > 0 && since.elapsed() >= pause_after)
        {
            stream.paused = true;
            stream.pending_since = None;
            writer.enqueue_line(format!("%pause %{pane_id}"));
            continue;
        }
        let mut line = match options.pause_after {
            Some(_) => format!("%extended-output %{pane_id} {age} : ").into_bytes(),
            None => format!("%output %{pane_id} ").into_bytes(),
        };
        for byte in bytes {
            if byte < b' ' || byte == b'\\' {
                line.extend_from_slice(format!("\\{byte:03o}").as_bytes());
            } else {
                line.push(byte);
            }
        }
        line.push(b'\n');
        writer.enqueue(&line);
        stream.offset = next_offset;
        if next_offset == end {
            stream.pending_since = None;
        }
    }
    Ok(())
}

fn write_control_notifications(
    writer: &mut ControlWriter,
    before: &ControlStateSnapshot,
    after: &ControlStateSnapshot,
) {
    let before_session_ids = before.sessions.keys().copied().collect::<BTreeSet<_>>();
    let after_session_ids = after.sessions.keys().copied().collect::<BTreeSet<_>>();
    for _ in before_session_ids.difference(&after_session_ids) {
        writer.enqueue_line("%sessions-changed");
    }
    if before.session_name != after.session_name {
        writer.enqueue_line(format!(
            "%session-renamed ${} {}",
            after.session_id, after.session_name
        ));
    }
    for session_id in before_session_ids.intersection(&after_session_ids) {
        if *session_id == after.session_id {
            continue;
        }
        let old = &before.sessions[session_id];
        let new = &after.sessions[session_id];
        if old != new {
            writer.enqueue_line(format!("%session-renamed ${session_id} {new}"));
        }
    }
    if before.active_window_id != after.active_window_id {
        writer.enqueue_line(format!(
            "%session-window-changed ${} @{}",
            after.session_id, after.active_window_id
        ));
    }

    let before_ids = before.windows.keys().copied().collect::<BTreeSet<_>>();
    let after_ids = after.windows.keys().copied().collect::<BTreeSet<_>>();
    for window_id in after_ids.difference(&before_ids) {
        writer.enqueue_line(format!("%window-add @{window_id}"));
    }
    for window_id in before_ids.intersection(&after_ids) {
        let old = &before.windows[window_id];
        let new = &after.windows[window_id];
        if old.active_pane_id != new.active_pane_id {
            writer.enqueue_line(format!(
                "%window-pane-changed @{} %{}",
                new.id, new.active_pane_id
            ));
        }
        if old.layout != new.layout {
            writer.enqueue_line(format!(
                "%layout-change @{} {} {} {}",
                new.id, new.layout, new.layout, new.flags
            ));
        }
        if old.name != new.name {
            writer.enqueue_line(format!("%window-renamed @{} {}", new.id, new.name));
        }
    }
    for window_id in before_ids.difference(&after_ids) {
        writer.enqueue_line(format!("%unlinked-window-close @{window_id}"));
    }

    let before_global_ids = before
        .global_windows
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let after_global_ids = after
        .global_windows
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    for window_id in after_global_ids.difference(&before_global_ids) {
        if !after.windows.contains_key(window_id) {
            writer.enqueue_line(format!("%unlinked-window-add @{window_id}"));
        }
    }
    for _ in after_session_ids.difference(&before_session_ids) {
        writer.enqueue_line("%sessions-changed");
    }
    for window_id in before_global_ids.difference(&after_global_ids) {
        if !before.windows.contains_key(window_id) {
            writer.enqueue_line(format!("%unlinked-window-close @{window_id}"));
        }
    }
    for window_id in before_global_ids.intersection(&after_global_ids) {
        let old = &before.global_windows[window_id];
        let new = &after.global_windows[window_id];
        let was_linked = before.windows.contains_key(window_id);
        let is_linked = after.windows.contains_key(window_id);
        if old.name != new.name && !is_linked {
            writer.enqueue_line(format!(
                "%unlinked-window-renamed @{window_id} {}",
                new.name
            ));
        }
        if was_linked == is_linked && old.links != new.links {
            let notification = match (new.links > old.links, is_linked) {
                (true, true) => "%window-add",
                (true, false) => "%unlinked-window-add",
                (false, true) => "%window-close",
                (false, false) => "%unlinked-window-close",
            };
            for _ in 0..old.links.abs_diff(new.links) {
                writer.enqueue_line(format!("{notification} @{window_id}"));
            }
        }
    }

    for (pane_id, old_mode) in &before.pane_modes {
        if after
            .pane_modes
            .get(pane_id)
            .is_some_and(|new_mode| new_mode != old_mode)
        {
            writer.enqueue_line(format!("%pane-mode-changed %{pane_id}"));
        }
    }

    for (name, data) in &after.buffers {
        if before
            .buffers
            .get(name)
            .is_some_and(|previous| previous != data)
        {
            writer.enqueue_line(format!("%paste-buffer-deleted {name}"));
        }
        if before.buffers.get(name) != Some(data) {
            writer.enqueue_line(format!("%paste-buffer-changed {name}"));
        }
    }
    for name in before.buffers.keys() {
        if !after.buffers.contains_key(name) {
            writer.enqueue_line(format!("%paste-buffer-deleted {name}"));
        }
    }

    for (name, (session_id, session_name)) in &after.clients {
        if before.clients.get(name) != Some(&(*session_id, session_name.clone())) {
            writer.enqueue_line(format!(
                "%client-session-changed {name} ${session_id} {session_name}"
            ));
        }
    }
    for name in before.clients.keys() {
        if !after.clients.contains_key(name) {
            writer.enqueue_line(format!("%client-detached {name}"));
        }
    }
}

fn write_all_control_layouts(writer: &mut ControlWriter, snapshot: &ControlStateSnapshot) {
    for window in snapshot.windows.values() {
        writer.enqueue_line(format!(
            "%layout-change @{} {} {} {}",
            window.id, window.layout, window.layout, window.flags
        ));
    }
}

fn apply_control_offset_actions(
    args: &[String],
    streams: &mut BTreeMap<u32, ControlPaneStream>,
    writer: &mut ControlWriter,
) -> bool {
    if args.first().map(String::as_str) != Some("refresh-client") {
        return false;
    }
    let mut handled = false;
    let mut index = 1;
    while index < args.len() {
        if args[index] != "-A" {
            index += 1;
            continue;
        }
        handled = true;
        let Some(value) = args.get(index + 1) else {
            break;
        };
        index += 2;
        let Some((pane, action)) = value.split_once(':') else {
            continue;
        };
        let Some(pane_id) = pane
            .strip_prefix('%')
            .and_then(|pane| pane.parse::<u32>().ok())
        else {
            continue;
        };
        let Some(stream) = streams.get_mut(&pane_id) else {
            continue;
        };
        match action {
            "off" => {
                stream.enabled = false;
                stream.offset = stream.observation.control_output_end();
                stream.pending_since = None;
            }
            "on" if !stream.enabled => {
                stream.enabled = true;
                stream.pending_since = Some(Instant::now());
            }
            "pause" if !stream.paused => {
                stream.paused = true;
                stream.pending_since = None;
                writer.enqueue_line(format!("%pause %{pane_id}"));
            }
            "continue" if stream.paused => {
                stream.paused = false;
                stream.offset = stream.observation.control_output_end();
                stream.pending_since = None;
                writer.enqueue_line(format!("%continue %{pane_id}"));
            }
            _ => {}
        }
    }
    handled
}

fn apply_control_subscription_actions(
    args: &[String],
    subscriptions: &mut ControlSubscriptions,
) -> bool {
    if args.first().map(String::as_str) != Some("refresh-client") {
        return false;
    }
    let mut handled = false;
    let mut index = 1;
    while index < args.len() {
        if args[index] != "-B" {
            index += 1;
            continue;
        }
        handled = true;
        let Some(value) = args.get(index + 1) else {
            break;
        };
        index += 2;
        let mut fields = value.splitn(3, ':');
        let name = fields.next().unwrap_or_default();
        let Some(what) = fields.next() else {
            subscriptions.entries.remove(name);
            subscriptions.reschedule(Instant::now());
            continue;
        };
        let Some(format) = fields.next() else {
            continue;
        };
        let kind = match what {
            "%*" => ControlSubscriptionKind::AllPanes,
            "@*" => ControlSubscriptionKind::AllWindows,
            value if value.starts_with('%') => value[1..]
                .parse()
                .map(ControlSubscriptionKind::Pane)
                .unwrap_or(ControlSubscriptionKind::Session),
            value if value.starts_with('@') => value[1..]
                .parse()
                .map(ControlSubscriptionKind::Window)
                .unwrap_or(ControlSubscriptionKind::Session),
            _ => ControlSubscriptionKind::Session,
        };
        subscriptions.entries.insert(
            name.to_string(),
            ControlSubscription {
                kind,
                format: format.to_string(),
                last: BTreeMap::new(),
            },
        );
        subscriptions.reschedule(Instant::now());
    }
    handled
}

#[allow(clippy::too_many_arguments)]
fn emit_control_subscription(
    name: &str,
    subscription: &mut ControlSubscription,
    target: ControlSubscriptionTarget,
    snapshot: &ControlStateSnapshot,
    cache: &mut status::RenderCache,
    state: &ServerState,
    stable_session: &str,
    cols: u16,
    rows: u16,
    writer: &mut ControlWriter,
) {
    let Some(value) = cache.expand_format_for_target(
        state,
        stable_session,
        target.window_id,
        target.pane_id,
        &subscription.format,
        cols,
        rows,
    ) else {
        return;
    };
    if subscription.last.get(&target) == Some(&value) {
        return;
    }
    let window = target
        .window_id
        .map_or_else(|| "-".to_string(), |id| format!("@{id}"));
    let index = target
        .window_index
        .map_or_else(|| "-".to_string(), |index| index.to_string());
    let pane = target
        .pane_id
        .map_or_else(|| "-".to_string(), |id| format!("%{id}"));
    writer.enqueue_line(format!(
        "%subscription-changed {name} ${} {window} {index} {pane} : {value}",
        snapshot.session_id
    ));
    subscription.last.insert(target, value);
}

#[allow(clippy::too_many_arguments)]
fn check_control_subscriptions(
    subscriptions: &mut ControlSubscriptions,
    cache: &mut status::RenderCache,
    state: &ServerState,
    stable_session: &str,
    cols: u16,
    rows: u16,
    writer: &mut ControlWriter,
) {
    let Some(snapshot) = state.control_snapshot(stable_session) else {
        return;
    };
    let session_target = ControlSubscriptionTarget {
        window_id: None,
        window_index: None,
        pane_id: None,
    };

    for (name, subscription) in &mut subscriptions.entries {
        if matches!(subscription.kind, ControlSubscriptionKind::Session) {
            emit_control_subscription(
                name,
                subscription,
                session_target,
                &snapshot,
                cache,
                state,
                stable_session,
                cols,
                rows,
                writer,
            );
        }
    }

    for (name, subscription) in &mut subscriptions.entries {
        let target = match subscription.kind {
            ControlSubscriptionKind::Pane(pane_id) => {
                snapshot.windows.values().find_map(|window| {
                    window
                        .panes
                        .iter()
                        .any(|pane| pane.id == pane_id)
                        .then_some(ControlSubscriptionTarget {
                            window_id: Some(window.id),
                            window_index: Some(window.index),
                            pane_id: Some(pane_id),
                        })
                })
            }
            ControlSubscriptionKind::Window(window_id) => {
                snapshot
                    .windows
                    .get(&window_id)
                    .map(|window| ControlSubscriptionTarget {
                        window_id: Some(window.id),
                        window_index: Some(window.index),
                        pane_id: None,
                    })
            }
            _ => None,
        };
        if let Some(target) = target {
            emit_control_subscription(
                name,
                subscription,
                target,
                &snapshot,
                cache,
                state,
                stable_session,
                cols,
                rows,
                writer,
            );
        }
    }

    let pane_targets = snapshot
        .windows
        .values()
        .flat_map(|window| {
            window.panes.iter().map(|pane| ControlSubscriptionTarget {
                window_id: Some(window.id),
                window_index: Some(window.index),
                pane_id: Some(pane.id),
            })
        })
        .collect::<Vec<_>>();
    for target in pane_targets {
        for (name, subscription) in &mut subscriptions.entries {
            if matches!(subscription.kind, ControlSubscriptionKind::AllPanes) {
                emit_control_subscription(
                    name,
                    subscription,
                    target,
                    &snapshot,
                    cache,
                    state,
                    stable_session,
                    cols,
                    rows,
                    writer,
                );
            }
        }
    }

    let window_targets = snapshot
        .windows
        .values()
        .map(|window| ControlSubscriptionTarget {
            window_id: Some(window.id),
            window_index: Some(window.index),
            pane_id: None,
        })
        .collect::<Vec<_>>();
    for target in window_targets {
        for (name, subscription) in &mut subscriptions.entries {
            if matches!(subscription.kind, ControlSubscriptionKind::AllWindows) {
                emit_control_subscription(
                    name,
                    subscription,
                    target,
                    &snapshot,
                    cache,
                    state,
                    stable_session,
                    cols,
                    rows,
                    writer,
                );
            }
        }
    }
}

enum ControlSizeAction {
    Client(u16, u16),
    Window(u32, Option<(u16, u16)>),
}

fn control_size_is_ignored(
    state: &ServerState,
    session_id: u32,
    client_name: &str,
    options: &ControlClientOptions,
) -> bool {
    options.ignore_size
        && state.attached_clients().iter().any(|client| {
            client.session_id == session_id
                && client.name != client_name
                && !client.ignore_size
                && (!client.control_mode || client.size_changed)
        })
}

fn is_control_refresh_operation(args: &[String]) -> bool {
    args.first().map(String::as_str) == Some("refresh-client")
        && args
            .iter()
            .any(|argument| matches!(argument.as_str(), "-c" | "-D" | "-L" | "-R" | "-S" | "-U"))
}

fn control_size_action(args: &[String]) -> Option<ControlSizeAction> {
    let value = args
        .iter()
        .position(|word| word == "-C")
        .and_then(|index| args.get(index + 1))?;
    if let Some(value) = value.strip_prefix('@') {
        let (window, size) = value.split_once(':')?;
        let window = window.parse().ok()?;
        if size.is_empty() {
            return Some(ControlSizeAction::Window(window, None));
        }
        let (cols, rows) = size.split_once('x').or_else(|| size.split_once(','))?;
        return Some(ControlSizeAction::Window(
            window,
            Some((cols.parse().ok()?, rows.parse().ok()?)),
        ));
    }
    let (cols, rows) = value.split_once(',').or_else(|| value.split_once('x'))?;
    Some(ControlSizeAction::Client(
        cols.parse().ok()?,
        rows.parse().ok()?,
    ))
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

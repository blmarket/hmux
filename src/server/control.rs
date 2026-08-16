//! Runtime-independent control-mode client state.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::integration::status::StatusHub;
use crate::tmux::message::{Frame, Message};
use hmux_rt::TaskHandle;

use super::attach::{self, ClientTty};
use super::command;
use super::command::queue::{CommandQueue, QueueCompletion};
use super::pane::{ControlOutputReader, OutputSubscription};
use super::registry::{self, Resolution};
use super::state::{
    ClientAction, ClientFlagState as ControlClientOptions, ClientRenderAttachment,
    ControlStateSnapshot, ServerState, SharedState,
};
use super::status;

const CONTROL_BUFFER_HIGH: usize = 8192;

/// How long a client that did not ask for `pause-after` may leave a pane's
/// output unread before the server stops serving it.
///
/// tmux's `CONTROL_MAXIMUM_AGE`. Holding the pane back for a client that has
/// stopped reading altogether would hold it back forever; tmux sends such a
/// client away instead, and so does this. What it measures is not quite tmux's:
/// tmux times the oldest block it has queued, which for a client that is being
/// held back at the journal's backlog can be old while the client is still
/// making progress. This times the last delivery instead, so only a stream that
/// has handed the client nothing at all for the whole window counts.
const CONTROL_MAXIMUM_AGE: Duration = Duration::from_secs(300);
const CLIENT_CONTROLCONTROL: i64 = 0x4000;

/// Whether the client keeps serving, or has reached the end of what it has to
/// serve and should push out what it queued.
///
/// This is a step's answer, not a phase the client is in: the sequence a
/// control client runs through is the shape of the task that drives it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlServing {
    Continue,
    Stop,
}

/// The native control-mode state machine without its blocking `poll(2)` loop.
///
/// Socket protocol I/O remains owned by the outer event-loop protocol actor.
/// This state owns only the descriptors passed during identification and the
/// native subscriptions needed by a control client.
pub(crate) struct EventControlClient {
    state: SharedState,
    hub: StatusHub,
    tasks: TaskHandle,
    client_tty: ClientTty,
    client_name: String,
    session_id: u32,
    stable_session: String,
    render_attachment: ClientRenderAttachment,
    output_fd: OwnedFd,
    control_writer: ControlWriter,
    subscriptions: ControlSubscriptions,
    format_cache: status::RenderCache,
    snapshot: ControlStateSnapshot,
    checkpoint: u64,
    streams: BTreeMap<u32, ControlPaneStream>,
    sequence: u64,
    pending: Vec<u8>,
    client_size: Option<(u16, u16)>,
    client_window_sizes: BTreeMap<u32, (u16, u16)>,
    command_queue: CommandQueue<ControlQueueItem>,
    background_commands: Vec<command::BackgroundCommandRequest>,
    stdin_open: bool,
    exit_status: i32,
    injected_prefix_pending: bool,
    context: command::ClientContext,
    options: ControlClientOptions,
    frames: VecDeque<Frame>,
}

impl EventControlClient {
    pub(crate) fn new(
        args: &[String],
        client_tty: ClientTty,
        state: SharedState,
        hub: StatusHub,
        context: &command::ClientContext,
        tasks: TaskHandle,
    ) -> io::Result<Self> {
        let session = match command::classify(args) {
            command::Intent::NewAttach => {
                let mut state = state.borrow_mut();
                command::new_session_for_attach(args, &mut state, context)
                    .map_err(io::Error::other)?
            }
            command::Intent::Attach => {
                let supplied_target = args
                    .iter()
                    .position(|arg| arg == "-t")
                    .and_then(|index| args.get(index + 1))
                    .map(String::as_str)
                    .or_else(|| {
                        args.iter().find_map(|arg| {
                            arg.strip_prefix("-t").filter(|value| !value.is_empty())
                        })
                    })
                    .map(|target| target.split(':').next().unwrap_or(target).to_string());
                let mut state = state.borrow_mut();
                let target = attach::attach_target(supplied_target, &mut state, context)
                    .map_err(io::Error::other)?;
                if state.find(&target).is_none() {
                    return Err(io::Error::other(format!("can't find session: {target}")));
                }
                if let Some(cwd) = command::command_value("attach-session", args, 'c') {
                    if let Some(session_id) = state.session_id(&target) {
                        state.set_session_cwd(session_id, Some(std::path::PathBuf::from(cwd)));
                    }
                }
                target
            }
            command::Intent::Command => "0".to_string(),
        };

        let (render_registry, session_id) = {
            let mut state = state.borrow_mut();
            let session_id = state
                .session_id(&session)
                .ok_or_else(|| io::Error::other(format!("can't find session: {session}")))?;
            state.touch_session_activity(session_id, true);
            (state.client_render_registry(), session_id)
        };
        let client_name = client_tty
            .tty_name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("client-{}", client_tty.client_pid.unwrap_or_default()));
        let options = control_client_options(args);
        let render_attachment = render_registry.attach_with_details(
            session_id,
            client_name.clone(),
            client_tty.term.clone().unwrap_or_default(),
            client_tty.client_pid,
            80,
            24,
            client_tty.flags,
            options.clone(),
            true,
        )?;
        let peer_uid = context.peer_uid;
        let peer_user = peer_uid
            .and_then(super::format::username)
            .unwrap_or_default();
        render_attachment.set_peer_identity(peer_uid, peer_user.clone());
        let stdin = client_tty
            .stdin
            .as_ref()
            .ok_or_else(|| io::Error::other("control client has no stdin"))?
            .as_raw_fd();
        set_nonblocking_fd(stdin)?;
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
        let duplicated_output = unsafe { libc::fcntl(output, libc::F_DUPFD_CLOEXEC, 0) };
        if duplicated_output < 0 {
            return Err(io::Error::last_os_error());
        }
        // `F_DUPFD_CLOEXEC` returned a new descriptor owned by this state.
        let output_fd = unsafe { OwnedFd::from_raw_fd(duplicated_output) };
        let mut control_writer = ControlWriter::new(output_fd.as_raw_fd())?;
        let stable_session = format!("${session_id}");
        let (snapshot, checkpoint) = {
            let state = state.borrow_mut();
            (
                state
                    .control_snapshot(&stable_session)
                    .ok_or_else(|| io::Error::other(format!("can't find session: {session}")))?,
                state.control_checkpoint_end(),
            )
        };
        let streams = control_pane_streams(&snapshot)?;
        let mut sequence = 0;
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
        for error in state.borrow_mut().take_config_errors() {
            control_writer.enqueue_line(format!("%config-error {error}"));
        }
        let format_cache = status::RenderCache::for_client(
            status::RenderClientContext {
                term: client_tty.term.clone(),
                tty: client_tty.tty_name.clone(),
                pid: client_tty.client_pid,
                uid: peer_uid,
                user: peer_user,
                cwd: context.cwd.clone(),
                control_mode: true,
                read_only: options.read_only,
                flags: options.display_flags(client_tty.flags),
                key_table: super::state::DEFAULT_KEY_TABLE.to_string(),
            },
            render_attachment.format_jobs(),
        );
        let mut control_context = context.clone();
        control_context.tty_name = Some(client_name.clone());
        control_context.current_session_id = Some(session_id);
        control_context.read_only = options.read_only;
        control_context.kind = command::ClientKind::Control {
            active_panes: options
                .active_pane
                .then(|| Rc::new(RefCell::new(BTreeMap::new()))),
        };
        let mut frames = VecDeque::new();
        frames.push_back(Frame::new(Message::Flags(
            options.client_flags(client_tty.flags),
        )));

        Ok(Self {
            state,
            hub,
            tasks,
            client_tty,
            client_name,
            session_id,
            stable_session,
            render_attachment,
            output_fd,
            control_writer,
            subscriptions: ControlSubscriptions::default(),
            format_cache,
            snapshot,
            checkpoint,
            streams,
            sequence,
            pending: Vec::new(),
            client_size: None,
            client_window_sizes: BTreeMap::new(),
            command_queue: CommandQueue::new(),
            background_commands: Vec::new(),
            stdin_open: true,
            exit_status: 0,
            injected_prefix_pending: false,
            context: control_context,
            options,
            frames,
        })
    }

    /// The descriptor the client's input arrives on.
    pub(crate) fn stdin_fd(&self) -> Option<BorrowedFd<'_>> {
        self.client_tty.stdin.as_ref().map(OwnedFd::as_fd)
    }

    /// The descriptor the client's control output goes out on.
    pub(crate) fn output_fd(&self) -> BorrowedFd<'_> {
        self.output_fd.as_fd()
    }

    /// The descriptor the server pokes when something this client renders from
    /// changed.
    pub(crate) fn state_fd(&self) -> BorrowedFd<'_> {
        // The descriptor is owned by the attachment for the returned borrow.
        unsafe { BorrowedFd::borrow_raw(self.render_attachment.as_raw_fd()) }
    }

    /// The panes this client streams output from, each with the descriptor its
    /// subscription notifies on.
    pub(crate) fn pane_fds(&self) -> Vec<(u32, BorrowedFd<'_>)> {
        self.streams
            .iter()
            .map(|(pane_id, stream)| {
                let fd = stream.subscription.as_raw_fd();
                // The subscription owns the descriptor for the returned borrow.
                (*pane_id, unsafe { BorrowedFd::borrow_raw(fd) })
            })
            .collect()
    }

    /// Whether the client still has input to read.
    pub(crate) fn stdin_open(&self) -> bool {
        self.stdin_open
    }

    /// Whether the writer still holds bytes the client has not taken.
    pub(crate) fn output_pending(&self) -> bool {
        self.control_writer.has_pending()
    }

    /// Hand the client's output descriptor whatever it will take right now.
    pub(crate) fn flush_output(&mut self) -> io::Result<()> {
        self.control_writer.flush()
    }

    /// Claim the render invalidation this client was notified about.
    pub(crate) fn note_state_change(&self) {
        let _ = self.render_attachment.take();
    }

    /// Claim one pane's output notification, reporting whether the pane is
    /// still one of this client's streams.
    pub(crate) fn note_pane_output(&mut self, pane_id: u32) -> bool {
        let Some(stream) = self.streams.get_mut(&pane_id) else {
            return false;
        };
        stream.subscription.drain();
        stream.pending_since.get_or_insert_with(Instant::now);
        true
    }

    pub(crate) fn deadline(&self, now: Instant) -> Option<Instant> {
        let subscription = self.subscriptions.next_check;
        let alert = self
            .state
            .borrow_mut()
            .alert_poll_timeout()
            .and_then(|duration| now.checked_add(duration));
        // A client that stopped reading has nothing else to wake it: without
        // this the backlog it holds would never be looked at again.
        let overrun = self
            .backlogged_streams()
            .filter_map(|stream| stream.delivered_at.checked_add(CONTROL_MAXIMUM_AGE))
            .min()
            .filter(|_| self.options.pause_after.is_none());
        [subscription, alert, overrun].into_iter().flatten().min()
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop_front()
    }

    pub(crate) fn take_background_commands(&mut self) -> Vec<command::BackgroundCommandRequest> {
        std::mem::take(&mut self.background_commands)
    }

    /// The frame reporting this client's exit status, sent once everything it
    /// queued has gone out.
    pub(crate) fn exit_frame(&self) -> Frame {
        Frame::new(Message::Exit(Some(self.exit_status)))
    }

    /// Move pane-output backlog into the writer and onto the socket until the
    /// backlog is gone or the socket stops taking bytes.
    ///
    /// One `write_control_output` pass moves at most a quarter of the writer's
    /// headroom per pane, so a single pass after a large burst would leave the
    /// rest of the journal stranded with no event to pump it: the pane only
    /// notifies on *new* output, and a fully flushed writer never reports
    /// writable. When the socket does fill, the writer keeps its pending block
    /// and the `Output` wait source resumes the pump on the next turn.
    ///
    /// This is what has to run last in a turn, since the two states it can end
    /// in are the only two a client may wait in: a socket that owes it a
    /// writable edge, or nothing left to send. A turn that ends anywhere else —
    /// a flush that empties the writer while the journal still holds output —
    /// parks a client with a backlog and nothing armed to wake it for.
    fn pump_output(&mut self) -> io::Result<()> {
        loop {
            write_control_output(&mut self.control_writer, &mut self.streams, &self.options)?;
            self.control_writer.flush()?;
            if self.control_writer.has_pending() || !self.streams_have_backlog() {
                return Ok(());
            }
        }
    }

    /// Whether any deliverable stream still has journal bytes the client has
    /// not been sent.
    fn streams_have_backlog(&self) -> bool {
        self.backlogged_streams().next().is_some()
    }

    /// The streams holding output this client has not been given yet. Each is
    /// keeping its pane's journal — and, past the journal's backlog, the pane
    /// itself — waiting for this client.
    fn backlogged_streams(&self) -> impl Iterator<Item = &ControlPaneStream> {
        let deliverable = !self.options.no_output;
        self.streams.values().filter(move |stream| {
            deliverable && stream.enabled && !stream.paused && !stream.reader.caught_up()
        })
    }

    /// Whether a stream has held a backlog for this client without delivering
    /// any of it for as long as tmux keeps a client that is too far behind.
    ///
    /// The client is done: it stopped reading, and the panes it holds cannot
    /// wait on it forever.
    fn too_far_behind(&self, now: Instant) -> bool {
        self.options.pause_after.is_none()
            && self.backlogged_streams().any(|stream| {
                now.saturating_duration_since(stream.delivered_at) >= CONTROL_MAXIMUM_AGE
            })
    }

    /// Give every pane back the output this client was holding, which is what
    /// its streams do when it goes away.
    fn release_streams(&mut self) {
        for stream in self.streams.values_mut() {
            stream.reader.detach();
        }
    }

    /// The housekeeping every turn starts with: what another client aimed at
    /// this one, and a session that may have gone away under it.
    pub(crate) fn refresh(&mut self) -> io::Result<ControlServing> {
        if self.handle_client_actions()? == ControlServing::Stop {
            return Ok(ControlServing::Stop);
        }
        self.refresh_session()
    }

    /// The tail of a turn: everything a client with an idle queue does, and
    /// then whether it has anything left to serve.
    pub(crate) fn finish_turn(
        &mut self,
        state_ready: bool,
        pane_ready: bool,
    ) -> io::Result<ControlServing> {
        if state_ready {
            self.advance_snapshot()?;
        }
        let alert_changed = {
            let mut state = self.state.borrow_mut();
            let changed = state.refresh_alerts(Instant::now());
            if changed {
                state.record_control_checkpoint();
            }
            changed
        };
        if alert_changed {
            self.advance_snapshot()?;
        }
        self.pump_output()?;
        if self.too_far_behind(Instant::now()) {
            self.release_streams();
            return Ok(ControlServing::Stop);
        }
        if pane_ready {
            {
                let mut state = self.state.borrow_mut();
                state.reap_exited_panes();
                state.record_control_checkpoint();
            }
            self.advance_snapshot()?;
        }
        if self
            .subscriptions
            .next_check
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.format_cache.update_agents(self.hub.snapshot());
            let (cols, rows) = self.client_size.unwrap_or((80, 24));
            let state = self.state.borrow_mut();
            check_control_subscriptions(
                &mut self.subscriptions,
                &mut self.format_cache,
                &state,
                &self.stable_session,
                cols,
                rows,
                &mut self.control_writer,
            );
            self.subscriptions.reschedule(Instant::now());
        }
        // Last, and a pump rather than a flush: the notifications above went
        // into the same writer, and emptying it here without refilling it from
        // the journal is what would leave the client parked on a socket it has
        // nothing queued for while a pane's output waits behind it.
        self.pump_output()?;
        // A queue with a current item is one this client is still running, so
        // an empty queue is the only point where closed input means the end.
        if !self.stdin_open && self.command_queue.is_empty() {
            return Ok(ControlServing::Stop);
        }
        Ok(ControlServing::Continue)
    }

    fn handle_client_actions(&mut self) -> io::Result<ControlServing> {
        for message in self.render_attachment.take_messages() {
            self.control_writer
                .enqueue_line(format!("%message {}", message.text));
        }
        self.apply_remote_flag_updates();
        let requested_switch = match self.render_attachment.take_action() {
            Some(ClientAction::Switch {
                session_id,
                destroyed,
            }) => Some((session_id, destroyed)),
            Some(ClientAction::Detach(_)) => return Ok(ControlServing::Stop),
            Some(ClientAction::Lock(command)) => {
                self.frames.push_back(Frame::new(Message::Lock(command)));
                None
            }
            Some(ClientAction::Suspend | ClientAction::SetSelection(_)) => None,
            Some(ClientAction::Keys(keys)) => {
                if attach::dispatch_control_client_keys(
                    &keys,
                    &mut self.injected_prefix_pending,
                    &self.state,
                    &self.stable_session,
                    &self.hub,
                    &self.context,
                    &mut self.background_commands,
                ) {
                    return Ok(ControlServing::Stop);
                }
                None
            }
            Some(ClientAction::Overlay { reply, .. } | ClientAction::Confirm { reply, .. }) => {
                if let Some(reply) = reply {
                    reply.send(Some(super::state::PromptCompletion {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit: 0,
                        inserted: false,
                    }));
                }
                None
            }
            None => None,
        };
        if let Some((session_id, destroyed)) = requested_switch {
            self.apply_switch(session_id, destroyed)?;
        }
        Ok(ControlServing::Continue)
    }

    /// Move this client onto `session_id`, reporting it as tmux's
    /// `server_client_set_session` does.
    fn apply_switch(&mut self, session_id: u32, destroyed: bool) -> io::Result<()> {
        let stable = format!("${session_id}");
        let checkpoint = {
            let state = self.state.borrow_mut();
            state
                .control_snapshot(&stable)
                .map(|_| state.control_checkpoint_end())
        };
        if let Some(checkpoint) = checkpoint {
            self.replace_session(session_id, stable, checkpoint, destroyed)?;
        }
        Ok(())
    }

    /// Adopt `refresh-client -f` values another client aimed at this one. The
    /// registry already recorded them, so this only keeps the client's own
    /// derived state (read-only routing, output policy, `%flags`) in step.
    fn apply_remote_flag_updates(&mut self) {
        let updates = self.render_attachment.take_flag_updates();
        if updates.is_empty() {
            return;
        }
        for value in &updates {
            self.options.apply_flags(value);
        }
        let display_flags = self.options.display_flags(self.client_tty.flags);
        self.render_attachment.update_control_flags(&self.options);
        self.format_cache
            .update_client_flags(display_flags, self.options.read_only);
        self.context.read_only = self.options.read_only;
        self.sync_active_pane_tracking();
        self.frames.push_back(Frame::new(Message::Flags(
            self.options.client_flags(self.client_tty.flags),
        )));
    }

    /// Bring the context's active-pane map in line with the client's current
    /// active-pane flag, keeping the existing map (and its recorded panes)
    /// when the flag stays on.
    fn sync_active_pane_tracking(&mut self) {
        let command::ClientKind::Control { active_panes } = &mut self.context.kind else {
            return;
        };
        if self.options.active_pane && active_panes.is_none() {
            *active_panes = Some(Rc::new(RefCell::new(BTreeMap::new())));
        } else if !self.options.active_pane {
            *active_panes = None;
        }
    }

    fn refresh_session(&mut self) -> io::Result<ControlServing> {
        let present = {
            let state = self.state.borrow_mut();
            state.control_snapshot(&self.stable_session).is_some()
        };
        if present {
            return Ok(ControlServing::Continue);
        }
        // Where a client goes when its session is destroyed is the server's
        // `detach-on-destroy` decision, delivered as a switch action before
        // this runs; reaching here means no session was offered.
        self.control_writer.enqueue_line("%sessions-changed");
        Ok(ControlServing::Stop)
    }

    fn replace_session(
        &mut self,
        session_id: u32,
        stable: String,
        checkpoint: u64,
        destroyed: bool,
    ) -> io::Result<()> {
        self.session_id = session_id;
        self.stable_session = stable;
        self.checkpoint = checkpoint;
        self.render_attachment.update_session(session_id);
        self.context.current_session_id = Some(session_id);
        if let Some(active_panes) = self.context.control_active_panes() {
            active_panes.borrow_mut().clear();
        }
        let next = self
            .state
            .borrow_mut()
            .control_snapshot(&self.stable_session)
            .ok_or_else(|| io::Error::other("fallback control session disappeared"))?;
        self.control_writer.enqueue_line(format!(
            "%session-changed ${} {}",
            next.session_id, next.session_name
        ));
        if destroyed {
            self.control_writer.enqueue_line("%sessions-changed");
            for window_id in self
                .snapshot
                .global_windows
                .keys()
                .filter(|window_id| !next.global_windows.contains_key(window_id))
            {
                self.control_writer
                    .enqueue_line(format!("%unlinked-window-close @{window_id}"));
            }
        }
        self.streams = control_pane_streams(&next)?;
        self.snapshot = next;
        Ok(())
    }

    /// Read what the client sent into the parse buffer.
    ///
    /// Reports whether the read budget ran out with the descriptor still
    /// readable: readiness is edge-triggered, so the caller has to come back
    /// rather than wait for an edge a fully drained descriptor never delivers.
    pub(crate) fn read_input(&mut self) -> io::Result<bool> {
        let fd = self
            .client_tty
            .stdin
            .as_ref()
            .ok_or_else(|| io::Error::other("control client stdin disappeared"))?
            .as_raw_fd();
        let mut buffer = [0u8; 4096];
        for _ in 0..16 {
            let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
            if count < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok(false);
                }
                return Err(error);
            }
            if count == 0 {
                self.stdin_open = false;
                return Ok(false);
            }
            self.pending.extend_from_slice(&buffer[..count as usize]);
        }
        Ok(true)
    }

    /// Turn whole lines the client sent into queued commands.
    pub(crate) fn parse_input(&mut self) -> io::Result<()> {
        while let Some(newline) = self.pending.iter().position(|byte| *byte == b'\n') {
            let raw_line = String::from_utf8_lossy(&self.pending[..newline]).to_string();
            self.pending.drain(..=newline);
            if raw_line.is_empty() {
                self.stdin_open = false;
                self.pending.clear();
                break;
            }
            let line = raw_line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            let aliases = self.state.borrow_mut().command_aliases();
            match command::command_string_groups_with_aliases(&line, &aliases) {
                Ok(groups) if !groups.is_empty() => self.command_queue.push_back_group(
                    groups
                        .into_iter()
                        .map(ControlQueueItem::Command)
                        .collect::<Vec<_>>(),
                ),
                Ok(_) => {}
                Err(result) => self
                    .command_queue
                    .push_back_group([ControlQueueItem::ParseError(result)]),
            }
        }
        Ok(())
    }

    /// Start queued commands until one has to run as a task, or the queue runs
    /// dry.
    ///
    /// An item that resolves without waiting is finished here; the one that
    /// cannot is handed back for its owner to await, which is what parks the
    /// rest of the queue behind it.
    pub(crate) fn start_next_command(&mut self) -> io::Result<Option<StartedControlCommand>> {
        loop {
            let Some(item) = self.command_queue.start_next() else {
                return Ok(None);
            };
            let argv = match item {
                ControlQueueItem::Completed(result) => {
                    let id = ControlCommandId::next(&mut self.sequence);
                    let flags = result.control_flags;
                    write_control_marker(&mut self.control_writer, "%begin", id, flags);
                    self.enqueue_command_result(&result, id, flags);
                    self.command_queue.complete(QueueCompletion::done());
                    continue;
                }
                ControlQueueItem::ParseError(result) => {
                    let id = ControlCommandId::next(&mut self.sequence);
                    write_control_marker(&mut self.control_writer, "%begin", id, 1);
                    self.control_writer.enqueue(b"parse error: ");
                    self.control_writer.enqueue(result.stderr.as_bytes());
                    write_control_marker(&mut self.control_writer, "%error", id, 1);
                    self.command_queue.complete(QueueCompletion::failed());
                    continue;
                }
                ControlQueueItem::Command(argv) => argv,
            };
            if let Some(started) = self.run_control_command(argv)? {
                return Ok(Some(started));
            }
        }
    }

    fn run_control_command(
        &mut self,
        argv: Vec<String>,
    ) -> io::Result<Option<StartedControlCommand>> {
        let id = ControlCommandId::next(&mut self.sequence);
        write_control_marker(&mut self.control_writer, "%begin", id, 1);
        let refresh_flags = control_refresh_flag_values(&argv);
        let reset_output_offsets = refresh_flags.iter().any(|value| {
            value
                .split(',')
                .any(|flag| flag.strip_prefix('!').unwrap_or(flag) == "no-output")
        });
        for value in &refresh_flags {
            self.options.apply_flags(value);
        }
        let switch_read_only = argv.first().is_some_and(|name| {
            matches!(registry::resolve(name), Resolution::Name("switch-client"))
        }) && control_command_has_flag(&argv, 'r');
        if switch_read_only {
            let enable = !self.options.read_only;
            self.options.read_only = enable;
            self.options.ignore_size = enable;
        }
        if !refresh_flags.is_empty() || switch_read_only {
            let display_flags = self.options.display_flags(self.client_tty.flags);
            self.render_attachment.update_control_flags(&self.options);
            self.format_cache
                .update_client_flags(display_flags, self.options.read_only);
            self.context.read_only = self.options.read_only;
            self.sync_active_pane_tracking();
            if reset_output_offsets {
                // `no-output` was turned on or off. Either way the client
                // starts again at what the pane has written by now, and a
                // stream detached while the flag was on takes its place in the
                // journal back.
                for stream in self.streams.values_mut() {
                    stream.reader.resume_at_end();
                    stream.pending_since = None;
                }
            }
            self.frames.push_back(Frame::new(Message::Flags(
                self.options.client_flags(self.client_tty.flags),
            )));
        }
        let handled_colour = apply_control_colour_report(&argv, &mut self.state.borrow_mut())?;
        let handled_offsets =
            apply_control_offset_actions(&argv, &mut self.streams, &mut self.control_writer);
        let handled_subscriptions =
            apply_control_subscription_actions(&argv, &mut self.subscriptions);
        if handled_offsets || handled_subscriptions {
            self.complete_local(id);
            return Ok(None);
        }
        if let Some(size) = control_size_action(&argv) {
            let next = {
                let mut state = self.state.borrow_mut();
                match size {
                    ControlSizeAction::Client(cols, rows) => {
                        self.client_size = Some((cols, rows));
                        self.render_attachment.update_size(cols, rows);
                        if !control_size_is_ignored(
                            &state,
                            self.session_id,
                            &self.client_name,
                            &self.options,
                        ) {
                            let _ = state.resize_session(&self.stable_session, cols, rows);
                            for (window_id, (cols, rows)) in &self.client_window_sizes {
                                let _ = state.resize_linked_window(
                                    &format!("@{window_id}"),
                                    *cols,
                                    *rows,
                                );
                            }
                        }
                    }
                    ControlSizeAction::Window(window_id, Some((cols, rows))) => {
                        self.client_window_sizes.insert(window_id, (cols, rows));
                        self.render_attachment.mark_size_changed();
                        if !control_size_is_ignored(
                            &state,
                            self.session_id,
                            &self.client_name,
                            &self.options,
                        ) {
                            let _ =
                                state.resize_linked_window(&format!("@{window_id}"), cols, rows);
                        }
                    }
                    ControlSizeAction::Window(window_id, None) => {
                        self.client_window_sizes.remove(&window_id);
                        let (cols, rows) = self.client_size.unwrap_or((80, 24));
                        self.render_attachment.mark_size_changed();
                        if !control_size_is_ignored(
                            &state,
                            self.session_id,
                            &self.client_name,
                            &self.options,
                        ) {
                            let _ =
                                state.resize_linked_window(&format!("@{window_id}"), cols, rows);
                        }
                    }
                }
                state.control_snapshot(&self.stable_session)
            };
            write_control_marker(&mut self.control_writer, "%end", id, 1);
            if let Some(next) = next {
                write_all_control_layouts(&mut self.control_writer, &next);
                sync_control_pane_streams(&next, &mut self.streams)?;
                self.snapshot = next;
            }
            self.command_queue.complete(QueueCompletion::done());
            return Ok(None);
        }
        if is_control_refresh_operation(&argv) || handled_colour || !refresh_flags.is_empty() {
            self.complete_local(id);
            return Ok(None);
        }
        if switch_read_only {
            write_control_marker(&mut self.control_writer, "%end", id, 1);
            self.control_writer.enqueue_line(format!(
                "%session-changed ${} {}",
                self.snapshot.session_id, self.snapshot.session_name
            ));
            self.command_queue.complete(QueueCompletion::done());
            return Ok(None);
        }
        let agents = self.hub.snapshot().panes;
        let queued =
            match command::start_resumable_command(&argv, &self.state, &agents, &self.context)
                .and_then(|queue| {
                    command::spawn_queue(&self.tasks, queue, Rc::clone(&self.state), 64)
                        .map_err(|error| command::CommandResult::err(format!("{error}\n")))
                }) {
                Ok(queued) => queued,
                Err(result) => {
                    self.finish_control_command(id, argv, result)?;
                    return Ok(None);
                }
            };
        Ok(Some(StartedControlCommand {
            id,
            argv,
            task: queued,
        }))
    }

    /// Report what a started command produced, and unpark the queue behind it.
    pub(crate) fn finish_command(
        &mut self,
        started: StartedControlCommand,
        result: io::Result<command::CommandResult>,
    ) -> io::Result<()> {
        let StartedControlCommand { id, argv, .. } = started;
        self.finish_control_command(id, argv, result?)
    }

    fn finish_control_command(
        &mut self,
        id: ControlCommandId,
        argv: Vec<String>,
        mut result: command::CommandResult,
    ) -> io::Result<()> {
        self.background_commands
            .append(&mut result.background_commands);
        let switches_client = argv.first().is_some_and(|name| {
            matches!(registry::resolve(name), Resolution::Name("switch-client"))
        });
        if result.exit == 0
            && matches!(
                argv.first().map(String::as_str),
                Some("select-window" | "selectw")
            )
        {
            if let (Some((cols, rows)), Some(target)) = (
                self.client_size,
                argv.iter()
                    .position(|word| word == "-t")
                    .and_then(|index| argv.get(index + 1)),
            ) {
                let _ = self
                    .state
                    .borrow_mut()
                    .resize_linked_window(target, cols, rows);
            }
        }
        self.enqueue_command_result(&result, id, 1);
        self.enqueue_config_errors()?;
        if result.exit != 0 || !switches_client {
            self.advance_snapshot()?;
            self.pump_output()?;
        } else {
            // tmux switches the client inside `switch-client` itself, so the
            // session is already the new one by the time the next queued
            // command is dispatched and `%session-changed` precedes its
            // `%begin`. Claiming the action here instead of leaving it for the
            // next event-loop pass keeps that order, which a client that
            // pipelines a command behind the switch can otherwise observe.
            if let Some((session_id, destroyed)) = self.render_attachment.take_switch() {
                self.apply_switch(session_id, destroyed)?;
            }
        }
        let completion = QueueCompletion {
            discard_group_tail: result.exit != 0 && !result.continue_queue,
            insert_next: std::mem::take(&mut result.inserted_results)
                .into_iter()
                .map(|result| vec![ControlQueueItem::Completed(result)])
                .collect(),
        };
        self.command_queue.complete(completion);
        Ok(())
    }

    fn complete_local(&mut self, id: ControlCommandId) {
        write_control_marker(&mut self.control_writer, "%end", id, 1);
        self.command_queue.complete(QueueCompletion::done());
    }

    fn enqueue_command_result(
        &mut self,
        result: &command::CommandResult,
        id: ControlCommandId,
        flags: u8,
    ) {
        if !result.stdout_data().is_empty() {
            self.control_writer.enqueue(result.stdout_data());
        }
        if !result.stderr.is_empty() {
            self.control_writer.enqueue(result.stderr.as_bytes());
        }
        if result.exit == 0 {
            write_control_marker(&mut self.control_writer, "%end", id, flags);
        } else {
            self.exit_status = result.exit;
            write_control_marker(&mut self.control_writer, "%error", id, flags);
        }
    }

    fn enqueue_config_errors(&mut self) -> io::Result<()> {
        for error in self.state.borrow_mut().take_config_errors() {
            self.control_writer
                .enqueue_line(format!("%config-error {error}"));
        }
        Ok(())
    }

    fn advance_snapshot(&mut self) -> io::Result<()> {
        advance_control_snapshot(
            &self.state,
            self.session_id,
            &self.stable_session,
            &self.client_name,
            &mut self.checkpoint,
            &mut self.snapshot,
            &mut self.streams,
            &mut self.control_writer,
        )
    }
}

fn set_nonblocking_fd(fd: i32) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
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
}

enum ControlQueueItem {
    Command(Vec<String>),
    ParseError(command::CommandResult),
    Completed(command::CommandResult),
}

/// One control command running as a task, with what its client needs to report
/// the result under.
pub(crate) struct StartedControlCommand {
    id: ControlCommandId,
    argv: Vec<String>,
    pub(crate) task: command::QueuedCommand,
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
    /// This client's place in the pane's output journal. The registration is
    /// what keeps the pane from discarding output the client has not been
    /// given yet, so a stream that stops delivering gives it back.
    reader: ControlOutputReader,
    subscription: OutputSubscription,
    enabled: bool,
    paused: bool,
    pending_since: Option<Instant>,
    /// When this stream last handed the client any of the pane's output. A
    /// stream with a backlog that has not moved for [`CONTROL_MAXIMUM_AGE`] is
    /// one whose client stopped reading.
    delivered_at: Instant,
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
    fn reschedule(&mut self, now: Instant) {
        self.next_check = (!self.entries.is_empty()).then(|| now + Duration::from_secs(1));
    }
}

fn control_client_options(args: &[String]) -> ControlClientOptions {
    let mut options = ControlClientOptions::from_attach_args(args);
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

fn apply_control_colour_report(args: &[String], state: &mut ServerState) -> io::Result<bool> {
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
    let _ = state.report_pane_control_colour(pane, report.as_bytes());
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
        let subscription = pane.observation.subscribe_output()?;
        streams.insert(
            pane.id,
            ControlPaneStream {
                runtime_id: pane.runtime_id,
                reader: ControlOutputReader::at_end(Rc::clone(&pane.observation)),
                subscription,
                enabled: true,
                paused: false,
                pending_since: None,
                delivered_at: Instant::now(),
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
                // A pane the client is only now learning about may already
                // have written: it starts at the pane's recent history rather
                // than at whatever it happens to have reached now.
                reader: ControlOutputReader::at_history(Rc::clone(&pane.observation)),
                subscription: pane.observation.subscribe_output()?,
                enabled: true,
                paused: false,
                pending_since: Some(Instant::now()),
                delivered_at: Instant::now(),
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn advance_control_snapshot(
    state: &SharedState,
    session_id: u32,
    stable_session: &str,
    client_name: &str,
    checkpoint: &mut u64,
    snapshot: &mut ControlStateSnapshot,
    streams: &mut BTreeMap<u32, ControlPaneStream>,
    writer: &mut ControlWriter,
) -> io::Result<()> {
    let (end, mut updates, current) = {
        let state = state.borrow_mut();
        let (end, updates) = state.control_checkpoints_since(session_id, *checkpoint);
        (end, updates, state.control_snapshot(stable_session))
    };
    *checkpoint = end;
    if let Some(current) = current {
        updates.push(current);
    }
    for next in updates {
        write_control_notifications(writer, client_name, snapshot, &next);
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
        // A client that asked for no output holds nothing back: tmux leaves it
        // out of the offsets a pane's buffer is kept for.
        for stream in streams.values_mut() {
            stream.reader.detach();
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
        let (next_offset, end, bytes) = stream.reader.chunk(raw_limit.max(1));
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
            // A paused pane keeps producing; the client resumes at whatever
            // the pane has written by the time it says `continue`, so its
            // place in the journal is given back rather than held here.
            stream.reader.detach();
            writer.enqueue_line(format!("%pause %{pane_id}"));
            continue;
        }
        let mut line = match options.pause_after {
            Some(_) => format!("%extended-output %{pane_id} {age} : ").into_bytes(),
            None => format!("%output %{pane_id} ").into_bytes(),
        };
        escape_control_output(&mut line, &bytes);
        line.push(b'\n');
        writer.enqueue(&line);
        stream.reader.advance(next_offset);
        stream.delivered_at = Instant::now();
        if next_offset == end {
            stream.pending_since = None;
        }
    }
    Ok(())
}

/// Append a pane's bytes to an `%output` line the way tmux's `control_write`
/// does: everything below a space, and the backslash itself, as a three-digit
/// octal escape.
///
/// The runs between escapes are copied whole. A flood is almost all printable,
/// and appending it a byte at a time is what made a large `%output` cost more
/// in the server than it did in the client reading it.
fn escape_control_output(line: &mut Vec<u8>, bytes: &[u8]) {
    line.reserve(bytes.len());
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte >= b' ' && *byte != b'\\' {
            continue;
        }
        line.extend_from_slice(&bytes[start..index]);
        line.extend_from_slice(&[
            b'\\',
            b'0' + (byte >> 6),
            b'0' + ((byte >> 3) & 0o7),
            b'0' + (byte & 0o7),
        ]);
        start = index + 1;
    }
    line.extend_from_slice(&bytes[start..]);
}

fn write_control_notifications(
    writer: &mut ControlWriter,
    client_name: &str,
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
        // A client hears about its own move as `%session-changed`, raised
        // where the move is applied; `%client-session-changed` is what the
        // other clients get. tmux draws the same line in
        // `control_notify_client_session_changed`.
        if name == client_name {
            continue;
        }
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
                stream.reader.detach();
                stream.pending_since = None;
            }
            "on" if !stream.enabled => {
                stream.enabled = true;
                stream.reader.resume();
                stream.pending_since = Some(Instant::now());
            }
            "pause" if !stream.paused => {
                stream.paused = true;
                stream.reader.detach();
                stream.pending_since = None;
                writer.enqueue_line(format!("%pause %{pane_id}"));
            }
            "continue" if stream.paused => {
                stream.paused = false;
                stream.reader.resume_at_end();
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
    options.ignore_size && state.other_client_constrains_size(session_id, client_name)
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
#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    use crate::integration::status::AgentStatus;
    use crate::integration::AgentState;
    use crate::observability::v1::PaneId;

    use super::super::state::PaneSpec;
    use super::*;

    /// The escaping `%output` applies, byte for byte as tmux's
    /// `control_write` spells it: a three-digit octal escape below a space and
    /// for the backslash itself, everything else through untouched.
    #[test]
    fn control_output_escapes_only_what_tmux_escapes() {
        let mut line = Vec::new();
        escape_control_output(&mut line, b"plain\r\n\\ \x7f\xc3\xa9\0end");
        assert_eq!(line, b"plain\\015\\012\\134 \x7f\xc3\xa9\\000end".to_vec());
    }

    fn drain(stream: &mut UnixStream) -> io::Result<String> {
        let mut output = Vec::new();
        loop {
            let mut buffer = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => output.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }
        Ok(String::from_utf8(output).expect("control output must be UTF-8"))
    }

    /// One turn of what the client's task does with it: claim what changed
    /// under it, run what it parsed, and do the work an idle queue leaves.
    fn turn(control: &mut EventControlClient, read_input: bool) -> io::Result<()> {
        assert_eq!(control.refresh()?, ControlServing::Continue);
        if read_input {
            control.read_input()?;
        }
        control.parse_input()?;
        assert!(
            control.start_next_command()?.is_none(),
            "this client's commands all finish where they are started"
        );
        assert_eq!(control.finish_turn(false, false)?, ControlServing::Continue);
        Ok(())
    }

    #[test]
    fn control_subscription_observes_agent_status_changes() -> io::Result<()> {
        let mut server = ServerState::empty();
        server.create_session("work", PaneSpec::Inert)?;
        let state = crate::server::state::shared_state(server);
        let hub = StatusHub::new();

        let (client_input, mut command_input) = UnixStream::pair()?;
        let (client_output, mut command_output) = UnixStream::pair()?;
        command_output.set_nonblocking(true)?;

        let mut tty = ClientTty::new();
        tty.stdin = Some(client_input.into());
        tty.stdout = Some(client_output.into());
        tty.flags = 0x2000;
        let args = [
            "attach-session".to_string(),
            "-t".to_string(),
            "work".to_string(),
        ];
        // The control client submits its command work to a loop, the same as
        // it does in the daemon; this test never suspends, so the loop only
        // has to exist.
        let runtime = hmux_rt::TaskRuntime::new()?;
        let mut control = EventControlClient::new(
            &args,
            tty,
            state,
            hub.clone(),
            &command::ClientContext::default(),
            runtime.handle(),
        )?;

        command_input.write_all(b"refresh-client -B 'agent:%*:#{pane_agent_state}'\n")?;
        turn(&mut control, true)?;
        control.subscriptions.next_check = Some(Instant::now());
        turn(&mut control, false)?;
        let initial = drain(&mut command_output)?;
        assert!(
            initial.contains("%subscription-changed agent $0 @0 0 %0 : none"),
            "unexpected initial control output: {initial:?}"
        );

        hub.publish(
            PaneId(0),
            AgentStatus {
                agent: "codex",
                pid: None,
                session_id: None,
                model: None,
                state: AgentState::Working,
            },
        );
        control.subscriptions.next_check = Some(Instant::now());
        turn(&mut control, false)?;
        let changed = drain(&mut command_output)?;
        assert!(
            changed.contains("%subscription-changed agent $0 @0 0 %0 : working"),
            "unexpected changed control output: {changed:?}"
        );

        Ok(())
    }
}

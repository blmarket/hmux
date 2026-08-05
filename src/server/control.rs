//! Runtime-independent control-mode client state.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::integration::status::StatusHub;
use crate::tmux::message::{Frame, Message};

use super::attach::{self, ClientTty};
use super::command;
use super::command::queue::{CommandQueue, QueueCompletion, QueueState, QueueTicket};
use super::pane::{NativePaneObservation, OutputSubscription};
use super::registry::{self, Resolution};
use super::state::{
    ClientAction, ClientFlagState as ControlClientOptions, ClientRenderAttachment,
    ControlStateSnapshot, ServerState, SharedState,
};
use super::status;
use super::task::{ReadySet, TaskState};

const CONTROL_BUFFER_HIGH: usize = 8192;
const CLIENT_CONTROLCONTROL: i64 = 0x4000;

/// One readiness source owned by an event-loop control client.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EventControlSource {
    Input,
    Output,
    State,
    Command,
    Pane(u32),
}

/// The native control-mode state machine without its blocking `poll(2)` loop.
///
/// Socket protocol I/O remains owned by the outer event-loop protocol actor.
/// This state owns only the descriptors passed during identification and the
/// native subscriptions needed by a control client.
pub(crate) struct EventControlClient {
    state: SharedState,
    hub: StatusHub,
    command_runtime: Rc<dyn command::CommandRuntime>,
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
    input_continuation: bool,
    client_size: Option<(u16, u16)>,
    client_window_sizes: BTreeMap<u32, (u16, u16)>,
    command_queue: CommandQueue<ControlQueueItem>,
    command_state: ControlCommandState,
    command_continuation: bool,
    background_commands: Vec<command::BackgroundCommandRequest>,
    stdin_open: bool,
    exit_status: i32,
    injected_prefix_pending: bool,
    context: command::ClientContext,
    options: ControlClientOptions,
    frames: VecDeque<Frame>,
    lifecycle: ControlLifecycle,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ControlLifecycle {
    Running,
    Draining,
    Finished,
}

enum ControlCommandState {
    Idle,
    Running(ActiveControlCommand),
    Waiting(ActiveControlCommand),
}

impl EventControlClient {
    pub(crate) fn new(
        args: &[String],
        client_tty: ClientTty,
        state: SharedState,
        hub: StatusHub,
        context: &command::ClientContext,
        command_runtime: Rc<dyn command::CommandRuntime>,
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
            options.display_flags(client_tty.flags),
            client_tty.flags,
            options.clone(),
            true,
        )?;
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
        for error in state
            .borrow_mut()
            .take_config_errors()
        {
            control_writer.enqueue_line(format!("%config-error {error}"));
        }
        let format_cache = status::RenderCache::for_client(
            status::RenderClientContext {
                term: client_tty.term.clone(),
                tty: client_tty.tty_name.clone(),
                pid: client_tty.client_pid,
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
            command_runtime,
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
            input_continuation: false,
            client_size: None,
            client_window_sizes: BTreeMap::new(),
            command_queue: CommandQueue::new(),
            command_state: ControlCommandState::Idle,
            command_continuation: false,
            background_commands: Vec::new(),
            stdin_open: true,
            exit_status: 0,
            injected_prefix_pending: false,
            context: control_context,
            options,
            frames,
            lifecycle: ControlLifecycle::Running,
        })
    }

    pub(crate) fn sources(&self) -> Vec<EventControlSource> {
        if self.lifecycle == ControlLifecycle::Finished {
            return Vec::new();
        }
        if self.lifecycle == ControlLifecycle::Draining {
            return self
                .control_writer
                .has_pending()
                .then_some(EventControlSource::Output)
                .into_iter()
                .collect();
        }
        let mut sources = Vec::with_capacity(4 + self.streams.len());
        if self.stdin_open {
            sources.push(EventControlSource::Input);
        }
        if self.control_writer.has_pending() {
            sources.push(EventControlSource::Output);
        }
        match self.command_state {
            ControlCommandState::Waiting(_) => sources.push(EventControlSource::Command),
            ControlCommandState::Idle => {
                sources.push(EventControlSource::State);
                sources.extend(self.streams.keys().copied().map(EventControlSource::Pane));
            }
            ControlCommandState::Running(_) => {}
        }
        sources
    }

    pub(crate) fn source_fd(&self, source: EventControlSource) -> Option<BorrowedFd<'_>> {
        let fd = match source {
            EventControlSource::Input => self.client_tty.stdin.as_ref()?.as_raw_fd(),
            EventControlSource::Output => self.output_fd.as_raw_fd(),
            EventControlSource::State => self.render_attachment.as_raw_fd(),
            EventControlSource::Command => match &self.command_state {
                ControlCommandState::Waiting(active) => active.task.wait().and_then(|wait| {
                    wait.sources().first().map(|source| source.fd().as_raw_fd())
                })?,
                _ => return None,
            },
            EventControlSource::Pane(pane_id) => {
                self.streams.get(&pane_id)?.subscription.as_raw_fd()
            }
        };
        // Every descriptor is owned by a field in `self` for the returned borrow.
        Some(unsafe { BorrowedFd::borrow_raw(fd) })
    }

    pub(crate) fn source_is_writable(source: EventControlSource) -> bool {
        matches!(source, EventControlSource::Output)
    }

    pub(crate) fn deadline(&self, now: Instant) -> Option<Instant> {
        if self.lifecycle != ControlLifecycle::Running {
            return None;
        }
        match &self.command_state {
            ControlCommandState::Waiting(active) => {
                return active.task.wait().and_then(|wait| wait.deadline())
            }
            ControlCommandState::Running(_) => return None,
            ControlCommandState::Idle => {}
        }
        let subscription = self.subscriptions.next_check;
        let alert = self
            .state
            .borrow_mut()
            .alert_poll_timeout()
            .and_then(|duration| now.checked_add(duration));
        match (subscription, alert) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (left, right) => left.or(right),
        }
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop_front()
    }

    pub(crate) fn take_input_continuation(&mut self) -> bool {
        std::mem::take(&mut self.input_continuation)
    }

    pub(crate) fn take_command_continuation(&mut self) -> bool {
        std::mem::take(&mut self.command_continuation)
    }

    pub(crate) fn take_background_commands(&mut self) -> Vec<command::BackgroundCommandRequest> {
        std::mem::take(&mut self.background_commands)
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.lifecycle == ControlLifecycle::Finished
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
        !self.options.no_output
            && self.streams.values().any(|stream| {
                stream.enabled
                    && !stream.paused
                    && stream.offset != stream.observation.control_output_end()
            })
    }

    pub(crate) fn drive(&mut self, source: Option<EventControlSource>) -> io::Result<()> {
        if self.lifecycle == ControlLifecycle::Finished {
            return Ok(());
        }
        if self.lifecycle == ControlLifecycle::Draining {
            if matches!(source, Some(EventControlSource::Output)) {
                self.control_writer.flush()?;
            }
            self.finish_if_drained();
            return Ok(());
        }

        self.handle_client_actions()?;
        if self.lifecycle == ControlLifecycle::Draining {
            self.finish_if_drained();
            return Ok(());
        }
        self.refresh_session()?;
        if self.lifecycle == ControlLifecycle::Draining {
            self.finish_if_drained();
            return Ok(());
        }

        let mut state_ready = false;
        let mut pane_state_ready = false;
        match source {
            Some(EventControlSource::Input) => self.read_input()?,
            Some(EventControlSource::Output) => self.control_writer.flush()?,
            Some(EventControlSource::State)
                if matches!(self.command_state, ControlCommandState::Idle) =>
            {
                let _ = self.render_attachment.take();
                state_ready = true;
            }
            Some(EventControlSource::Command) => self.complete_pending_command(true)?,
            Some(EventControlSource::Pane(pane_id))
                if matches!(self.command_state, ControlCommandState::Idle) =>
            {
                if let Some(stream) = self.streams.get_mut(&pane_id) {
                    stream.subscription.drain();
                    stream.pending_since.get_or_insert_with(Instant::now);
                    pane_state_ready = true;
                }
            }
            None | Some(_) => {}
        }

        let pending_complete = match &mut self.command_state {
            ControlCommandState::Waiting(active) => active.task.poll(&ReadySet::default()),
            _ => false,
        };
        if source.is_none() && pending_complete {
            self.complete_pending_command(false)?;
        }
        self.parse_input()?;
        self.run_command_queue()?;
        if matches!(self.command_state, ControlCommandState::Idle) {
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
            if pane_state_ready {
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
        }
        self.control_writer.flush()?;
        if !self.stdin_open
            && !matches!(self.command_state, ControlCommandState::Waiting(_))
            && matches!(self.command_queue.state(), QueueState::Empty)
        {
            self.pump_output()?;
            self.lifecycle = ControlLifecycle::Draining;
            self.finish_if_drained();
        }
        Ok(())
    }

    fn handle_client_actions(&mut self) -> io::Result<()> {
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
            Some(ClientAction::Detach) => {
                self.lifecycle = ControlLifecycle::Draining;
                return Ok(());
            }
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
                    self.lifecycle = ControlLifecycle::Draining;
                    return Ok(());
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
        self.render_attachment
            .update_control_flags(display_flags.clone(), &self.options);
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

    fn refresh_session(&mut self) -> io::Result<()> {
        let replacement = {
            let state = self.state.borrow_mut();
            // Where a client goes when its session is destroyed is the server's
            // `detach-on-destroy` decision, delivered as a switch action before
            // this runs; reaching here means no session was offered.
            if state.control_snapshot(&self.stable_session).is_some() {
                None
            } else {
                self.control_writer.enqueue_line("%sessions-changed");
                self.lifecycle = ControlLifecycle::Draining;
                None
            }
        };
        if let Some((session_id, stable, checkpoint, destroyed)) = replacement {
            self.replace_session(session_id, stable, checkpoint, destroyed)?;
        }
        Ok(())
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

    fn read_input(&mut self) -> io::Result<()> {
        self.input_continuation = false;
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
                    return Ok(());
                }
                return Err(error);
            }
            if count == 0 {
                self.stdin_open = false;
                return Ok(());
            }
            self.pending.extend_from_slice(&buffer[..count as usize]);
        }
        // The budget ran out with the descriptor still readable. Readiness is
        // edge-triggered, so ask for another pass instead of waiting for an
        // edge that a fully drained descriptor would never deliver.
        self.input_continuation = true;
        Ok(())
    }

    fn parse_input(&mut self) -> io::Result<()> {
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
            let aliases = self
                .state
                .borrow_mut()
                .command_aliases();
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

    fn complete_pending_command(&mut self, source_ready: bool) -> io::Result<()> {
        let state = std::mem::replace(&mut self.command_state, ControlCommandState::Idle);
        let mut active = match state {
            ControlCommandState::Waiting(active) => active,
            state => {
                self.command_state = state;
                return Ok(());
            }
        };
        active
            .task
            .poll_after_single_source_wakeup(source_ready, Instant::now());
        self.command_state = ControlCommandState::Running(active);
        self.drive_active_command()
    }

    fn run_command_queue(&mut self) -> io::Result<()> {
        if matches!(self.command_state, ControlCommandState::Running(_)) {
            self.drive_active_command()?;
        }
        while matches!(self.command_state, ControlCommandState::Idle) {
            let Some(started) = self.command_queue.start_next() else {
                break;
            };
            let ticket = started.ticket;
            let argv = match started.value {
                ControlQueueItem::Completed(result) => {
                    let id = ControlCommandId::next(&mut self.sequence);
                    let flags = result.control_flags;
                    write_control_marker(&mut self.control_writer, "%begin", id, flags);
                    self.enqueue_command_result(&result, id, flags);
                    self.command_queue
                        .complete(ticket, QueueCompletion::done())
                        .map_err(|_| io::Error::other("stale completed control queue item"))?;
                    continue;
                }
                ControlQueueItem::ParseError(result) => {
                    let id = ControlCommandId::next(&mut self.sequence);
                    write_control_marker(&mut self.control_writer, "%begin", id, 1);
                    self.control_writer.enqueue(b"parse error: ");
                    self.control_writer.enqueue(result.stderr.as_bytes());
                    write_control_marker(&mut self.control_writer, "%error", id, 1);
                    self.command_queue
                        .complete(ticket, QueueCompletion::failed())
                        .map_err(|_| io::Error::other("stale control parse error"))?;
                    continue;
                }
                ControlQueueItem::Command(argv) => argv,
            };
            self.run_control_command(ticket, argv)?;
        }
        Ok(())
    }

    fn run_control_command(&mut self, ticket: QueueTicket, argv: Vec<String>) -> io::Result<()> {
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
            self.render_attachment
                .update_control_flags(display_flags.clone(), &self.options);
            self.format_cache
                .update_client_flags(display_flags, self.options.read_only);
            self.context.read_only = self.options.read_only;
            self.sync_active_pane_tracking();
            if reset_output_offsets {
                for stream in self.streams.values_mut() {
                    stream.offset = stream.observation.control_output_end();
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
            return self.complete_local(ticket, id);
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
            return self
                .command_queue
                .complete(ticket, QueueCompletion::done())
                .map_err(|_| io::Error::other("stale control queue item"));
        }
        if is_control_refresh_operation(&argv) || handled_colour || !refresh_flags.is_empty() {
            return self.complete_local(ticket, id);
        }
        if switch_read_only {
            write_control_marker(&mut self.control_writer, "%end", id, 1);
            self.control_writer.enqueue_line(format!(
                "%session-changed ${} {}",
                self.snapshot.session_id, self.snapshot.session_name
            ));
            return self
                .command_queue
                .complete(ticket, QueueCompletion::done())
                .map_err(|_| io::Error::other("stale control queue item"));
        }
        let agents = self.hub.snapshot().panes;
        self.command_queue
            .wait(ticket)
            .map_err(|_| io::Error::other("stale control queue wait"))?;
        let queue =
            match command::start_resumable_command(&argv, &self.state, &agents, &self.context) {
                Ok(queue) => queue,
                Err(result) => return self.finish_control_command(ticket, id, argv, result),
            };
        self.command_state = ControlCommandState::Running(ActiveControlCommand {
            ticket,
            id,
            argv,
            task: TaskState::new(command::CommandCoroutine::new(
                queue,
                Rc::clone(&self.state),
                Rc::clone(&self.command_runtime),
                64,
            )),
        });
        self.drive_active_command()
    }

    fn drive_active_command(&mut self) -> io::Result<()> {
        let (complete, has_source) = match &mut self.command_state {
            ControlCommandState::Running(active) => {
                let complete = active.task.poll(&ReadySet::default());
                let has_source = active
                    .task
                    .wait()
                    .is_some_and(|wait| !wait.sources().is_empty());
                (complete, has_source)
            }
            _ => return Ok(()),
        };
        if complete {
            let state = std::mem::replace(&mut self.command_state, ControlCommandState::Idle);
            let ControlCommandState::Running(mut active) = state else {
                return Ok(());
            };
            let result = active
                .task
                .take_output()
                .ok_or_else(|| io::Error::other("completed control command has no result"))??;
            self.finish_control_command(active.ticket, active.id, active.argv, result)
        } else if has_source {
            let state = std::mem::replace(&mut self.command_state, ControlCommandState::Idle);
            let ControlCommandState::Running(active) = state else {
                return Ok(());
            };
            self.command_state = ControlCommandState::Waiting(active);
            Ok(())
        } else {
            self.command_continuation = true;
            Ok(())
        }
    }

    fn finish_control_command(
        &mut self,
        ticket: QueueTicket,
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
        }
        let completion = QueueCompletion {
            discard_group_tail: result.exit != 0 && !result.continue_queue,
            insert_next: std::mem::take(&mut result.inserted_results)
                .into_iter()
                .map(|result| vec![ControlQueueItem::Completed(result)])
                .collect(),
        };
        self.command_queue
            .resume(ticket)
            .map_err(|_| io::Error::other("stale control command resume"))?;
        self.command_queue
            .complete(ticket, completion)
            .map_err(|_| io::Error::other("stale control command result"))
    }

    fn complete_local(&mut self, ticket: QueueTicket, id: ControlCommandId) -> io::Result<()> {
        write_control_marker(&mut self.control_writer, "%end", id, 1);
        self.command_queue
            .complete(ticket, QueueCompletion::done())
            .map_err(|_| io::Error::other("stale control queue item"))
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
        for error in self
            .state
            .borrow_mut()
            .take_config_errors()
        {
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
            &mut self.checkpoint,
            &mut self.snapshot,
            &mut self.streams,
            &mut self.control_writer,
        )
    }

    fn finish_if_drained(&mut self) {
        if self.lifecycle == ControlLifecycle::Draining && !self.control_writer.has_pending() {
            self.frames
                .push_back(Frame::new(Message::Exit(Some(self.exit_status))));
            self.lifecycle = ControlLifecycle::Finished;
        }
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

struct ActiveControlCommand {
    ticket: QueueTicket,
    id: ControlCommandId,
    argv: Vec<String>,
    task: TaskState<command::CommandCoroutine>,
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
    observation: Rc<NativePaneObservation>,
    subscription: OutputSubscription,
    enabled: bool,
    paused: bool,
    pending_since: Option<Instant>,
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
        let offset = pane.observation.control_output_end();
        let subscription = pane.observation.subscribe_output()?;
        streams.insert(
            pane.id,
            ControlPaneStream {
                runtime_id: pane.runtime_id,
                offset,
                observation: Rc::clone(&pane.observation),
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
                observation: Rc::clone(&pane.observation),
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
    state: &SharedState,
    session_id: u32,
    stable_session: &str,
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
        let event_loop = crate::event_loop::driver::EventLoop::new()?;
        let mut control = EventControlClient::new(
            &args,
            tty,
            state,
            hub.clone(),
            &command::ClientContext::default(),
            Rc::new(crate::event_loop::suspend::EventCommandRuntime::new(
                event_loop.executor_handle(),
            )),
        )?;

        command_input.write_all(b"refresh-client -B 'agent:%*:#{pane_agent_state}'\n")?;
        control.drive(Some(EventControlSource::Input))?;
        control.subscriptions.next_check = Some(Instant::now());
        control.drive(None)?;
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
        control.drive(None)?;
        let changed = drain(&mut command_output)?;
        assert!(
            changed.contains("%subscription-changed agent $0 @0 0 %0 : working"),
            "unexpected changed control output: {changed:?}"
        );

        Ok(())
    }
}

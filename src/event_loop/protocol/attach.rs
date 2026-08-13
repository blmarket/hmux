//! Event-loop ownership for interactive attach.
//!
//! The protocol actor owns the native attach session directly. Runtime frames,
//! tty readiness, pane-output notifications, and deadlines are represented as
//! bounded queues and reactor sources; no attach worker or bridge is involved.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
use std::rc::Rc;
use std::time::Instant;

use crate::integration::status::StatusHub;
use crate::server::attach::FrameSink;
use crate::server::attach::{
    self, AttachCommandContinuation, AttachDrive, AttachPrepared, AttachSession,
    AttachStartFailure, AttachWaitReady, AttachWaitSources, ClientTty,
};
use crate::server::command::{self, ClientContext};
use crate::server::state::SharedState;
use hmux_rt::WakeFn;
use crate::tmux::codec::{dup_fd, encode_bytes, MAX_IMSGSIZE};
use crate::tmux::message::Frame;
use crate::tmux::traits::NonblockingFrameReader;

use super::super::actor::ActorRef;
use super::super::driver::Outbox;
use super::super::job::JobEvent;
use super::client::{
    AttachClientState, CommandClientState, CommandOperation, ProtocolClient, ProtocolCloseReason,
    ProtocolEvent, ProtocolIoSide, ProtocolState,
};

const ATTACH_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;
const IMMEDIATE_TURN_BUDGET: usize = 64;
const COMMAND_QUEUE_BUDGET: usize = 64;

/// One source an attached client waits on, watched by the central reactor.
///
/// `Runtime` carries a generation because the reactor keys a registration by
/// its source, so a source whose descriptor is replaced has to become a new
/// key. `Command` has no descriptor — a suspension is waited on through a wake
/// — and its generation now only keeps a stale wake from being credited to a
/// newer suspension.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EventAttachSource {
    Runtime {
        source: AttachRuntimeSource,
        generation: u64,
    },
    Command(u64),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum AttachRuntimeSource {
    Control,
    Input,
    TtyOutput,
    Output,
    Prompt,
    Render,
    Status,
    PopupRead,
    PopupWrite,
}

/// An attach startup failure reported through the ordinary command response.
#[derive(Debug)]
struct AttachStartError {
    message: String,
}

impl AttachStartError {
    fn from_failure(failure: AttachStartFailure) -> Self {
        Self {
            message: failure.into_message(),
        }
    }

    fn into_message(self) -> String {
        self.message
    }
}

impl From<io::Error> for AttachStartError {
    fn from(error: io::Error) -> Self {
        Self {
            message: format!("{error}\n"),
        }
    }
}

struct AttachInput {
    frames: VecDeque<QueuedAttachFrame>,
    bytes: usize,
}

struct QueuedAttachFrame {
    frame: Frame,
    wire_len: usize,
}

impl AttachInput {
    fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            bytes: 0,
        }
    }

    fn push(&mut self, frame: Frame) {
        let wire_len = encode_bytes(&frame).len();
        self.bytes += wire_len;
        self.frames.push_back(QueuedAttachFrame { frame, wire_len });
    }

    fn pop(&mut self) -> Option<Frame> {
        let queued = self.frames.pop_front()?;
        self.bytes = self.bytes.saturating_sub(queued.wire_len);
        Some(queued.frame)
    }

    fn is_below_high_water(&self) -> bool {
        self.bytes < ATTACH_QUEUE_LIMIT
    }

    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl NonblockingFrameReader for AttachInput {
    fn try_recv(&mut self) -> io::Result<Frame> {
        self.pop()
            .ok_or_else(|| io::Error::from(io::ErrorKind::WouldBlock))
    }
}

struct AttachOutput {
    frames: VecDeque<Frame>,
    bytes: usize,
}

impl AttachOutput {
    fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            bytes: 0,
        }
    }

    fn pop(&mut self) -> Option<Frame> {
        let frame = self.frames.pop_front()?;
        self.bytes = self.bytes.saturating_sub(encode_bytes(&frame).len());
        Some(frame)
    }

    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl FrameSink for AttachOutput {
    fn send(&mut self, frame: Frame) -> io::Result<()> {
        let bytes = encode_bytes(&frame).len();
        if bytes > ATTACH_QUEUE_LIMIT.saturating_sub(self.bytes) {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "attach protocol output limit exceeded",
            ));
        }
        self.bytes += bytes;
        self.frames.push_back(frame);
        Ok(())
    }
}

/// Interactive attach state driven directly by one protocol actor.
pub(super) struct EventAttachClient {
    session: AttachSession,
    state: SharedState,
    hub: StatusHub,
    input: AttachInput,
    output: AttachOutput,
    runtime_sources: BTreeMap<EventAttachSource, OwnedFd>,
    runtime_desired: BTreeSet<EventAttachSource>,
    registrations: BTreeMap<AttachRuntimeSource, ((RawFd, u64), EventAttachSource)>,
    /// Hands out generations for the runtime sources, which each keep the one
    /// they were given in `registrations`.
    next_generation: u64,
    /// The generation of the in-flight suspension. Only one command runs at a
    /// time, so this single value is the whole of that source's identity,
    /// which is why it is a counter of its own rather than a ticket from
    /// `next_generation`.
    command_generation: u64,
    deadline: Option<Instant>,
    phase: AttachPhase,
    background_commands: Vec<command::BackgroundCommandRequest>,
    command_runtime: Rc<dyn command::CommandRuntime>,
}

struct ActiveAttachCommand {
    task: command::QueuedCommand,
    continuation: AttachCommandContinuation,
    allows_attach_io: bool,
}

enum AttachPhase {
    Session,
    Running(ActiveAttachCommand),
    Waiting(ActiveAttachCommand),
    Finished,
}

impl EventAttachClient {
    fn new(
        args: &[String],
        client_tty: ClientTty,
        state: SharedState,
        hub: StatusHub,
        context: &ClientContext,
        command_runtime: Rc<dyn command::CommandRuntime>,
    ) -> Result<Self, AttachStartError> {
        let mut output = AttachOutput::new();
        let session =
            attach::start_attach_session(args, client_tty, &state, &hub, context, &mut output)
                .map_err(AttachStartError::from_failure)?;
        let mut attach = Self {
            session,
            state,
            hub,
            input: AttachInput::new(),
            output,
            runtime_sources: BTreeMap::new(),
            runtime_desired: BTreeSet::new(),
            registrations: BTreeMap::new(),
            next_generation: 0,
            command_generation: 0,
            deadline: None,
            phase: AttachPhase::Session,
            background_commands: Vec::new(),
            command_runtime,
        };
        attach.refresh_wait()?;
        Ok(attach)
    }

    fn sources(&self) -> BTreeSet<EventAttachSource> {
        if matches!(self.phase, AttachPhase::Finished) {
            BTreeSet::new()
        } else {
            self.runtime_desired.clone()
        }
    }

    pub(super) fn source_fd(&self, source: EventAttachSource) -> Option<BorrowedFd<'_>> {
        match source {
            // A suspended command queue has no descriptor: it is waited on
            // through the wake its client installs.
            EventAttachSource::Command(_) => None,
            EventAttachSource::Runtime { .. } => self.runtime_sources.get(&source).map(AsFd::as_fd),
        }
    }

    /// Install the wake for the suspension this client's command queue is
    /// parked on, if `generation` still names it.
    pub(super) fn set_command_wake(&mut self, generation: u64, wake: &WakeFn) -> bool {
        match &mut self.phase {
            AttachPhase::Waiting(active) if generation == self.command_generation => {
                active.task.set_wake(wake);
                true
            }
            _ => false,
        }
    }

    pub(super) fn source_is_writable(source: EventAttachSource) -> bool {
        matches!(
            source,
            EventAttachSource::Runtime {
                source: AttachRuntimeSource::TtyOutput,
                ..
            } | EventAttachSource::Runtime {
                source: AttachRuntimeSource::PopupWrite,
                ..
            }
        )
    }

    fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.output.pop()
    }

    fn accepts_protocol_input(&self) -> bool {
        let accepts = match &self.phase {
            AttachPhase::Session => true,
            AttachPhase::Running(active) | AttachPhase::Waiting(active) => active.allows_attach_io,
            AttachPhase::Finished => false,
        };
        accepts && self.input.is_below_high_water()
    }

    fn is_finished(&self) -> bool {
        matches!(self.phase, AttachPhase::Finished) && self.output.is_empty()
    }

    fn take_background_commands(&mut self) -> Vec<command::BackgroundCommandRequest> {
        std::mem::take(&mut self.background_commands)
    }

    fn handle_frame(&mut self, frame: Frame) -> io::Result<()> {
        if matches!(self.phase, AttachPhase::Finished) {
            return Ok(());
        }
        self.input.push(frame);
        self.refresh_wait()
    }

    fn drive(&mut self, source: Option<EventAttachSource>) -> io::Result<()> {
        if let Some(EventAttachSource::Command(generation)) = source {
            // A readiness event that outlived its suspension names an older
            // generation; the descriptor it reported on is gone.
            if generation != self.command_generation {
                return Ok(());
            }
            self.complete_pending_command(true)?;
            return self.refresh_wait();
        }
        let ready = match source {
            Some(source @ EventAttachSource::Runtime { source: kind, .. }) => {
                if !self.runtime_desired.remove(&source) {
                    return Ok(());
                }
                let mut ready = AttachWaitReady::default();
                Self::mark_ready(&mut ready, kind);
                ready
            }
            None | Some(EventAttachSource::Command(_)) => AttachWaitReady::default(),
        };
        if source.is_some() {
            self.drive_session(ready)?;
        }
        self.refresh_wait()
    }

    pub(super) fn drive_timer(&mut self) -> io::Result<()> {
        self.deadline = None;
        if matches!(self.phase, AttachPhase::Waiting(_)) {
            self.complete_pending_command(false)?;
            return self.refresh_wait();
        }
        match &self.phase {
            AttachPhase::Running(_) => {
                self.drive_active_command()?;
            }
            AttachPhase::Session => self.drive_session(AttachWaitReady::default())?,
            AttachPhase::Waiting(_) | AttachPhase::Finished => {}
        }
        self.refresh_wait()
    }

    pub(super) fn shutdown(&mut self) {
        self.phase = AttachPhase::Finished;
        self.input.frames.clear();
        self.input.bytes = 0;
        self.output.frames.clear();
        self.output.bytes = 0;
        self.runtime_desired.clear();
        self.runtime_sources.clear();
        self.deadline = None;
    }

    fn drive_session(&mut self, ready: AttachWaitReady) -> io::Result<()> {
        match self.session.drive_ready(
            &self.state,
            &self.hub,
            ready,
            &mut self.input,
            &mut self.output,
        )? {
            AttachDrive::Continue => {}
            AttachDrive::Finished => self.finish(),
        }
        Ok(())
    }

    fn refresh_wait(&mut self) -> io::Result<()> {
        if matches!(self.phase, AttachPhase::Finished) {
            return Ok(());
        }
        // A command's own turn can defer the next one — a burst of mouse
        // reports in one read defers one command per report — so keep starting
        // and draining until the queue is dry. Stopping after one would leave
        // the rest waiting behind a `prepare_wait` that blocks on the tty while
        // a command is pending, so the tail of the burst never ran.
        for _ in 0..IMMEDIATE_TURN_BUDGET {
            if matches!(self.phase, AttachPhase::Session) {
                self.start_session_command()?;
            }
            if matches!(self.phase, AttachPhase::Running(_)) {
                self.drive_active_command()?;
            }
            if !matches!(self.phase, AttachPhase::Session) || !self.session.has_pending_command() {
                break;
            }
        }
        match &self.phase {
            AttachPhase::Waiting(active) if active.allows_attach_io => {
                self.refresh_session_sources()?;
                if matches!(self.phase, AttachPhase::Finished) {
                    return Ok(());
                }
                self.runtime_desired
                    .insert(EventAttachSource::Command(self.command_generation));
                Ok(())
            }
            AttachPhase::Waiting(_) => {
                self.runtime_desired.clear();
                self.runtime_desired
                    .insert(EventAttachSource::Command(self.command_generation));
                // A parked queue has no deadline of its own; its wake is what
                // brings this client back.
                self.deadline = None;
                Ok(())
            }
            AttachPhase::Running(_) => {
                self.runtime_desired.clear();
                self.deadline = Some(Instant::now());
                Ok(())
            }
            AttachPhase::Session => self.refresh_session_sources(),
            AttachPhase::Finished => Ok(()),
        }
    }

    fn refresh_session_sources(&mut self) -> io::Result<()> {
        for _ in 0..IMMEDIATE_TURN_BUDGET {
            match self
                .session
                .prepare_wait(&self.state, -1, !self.input.is_empty())?
            {
                AttachPrepared::Ready(ready) => self.drive_session(ready)?,
                AttachPrepared::Wait { sources, deadline } => {
                    self.apply_wait(sources, deadline)?;
                    return Ok(());
                }
                AttachPrepared::Finished => {
                    self.finish();
                    return Ok(());
                }
            }
            if matches!(self.phase, AttachPhase::Finished) {
                return Ok(());
            }
        }

        self.runtime_desired.clear();
        self.runtime_sources.clear();
        self.deadline = Some(Instant::now());
        Ok(())
    }

    fn start_session_command(&mut self) -> io::Result<()> {
        let Some(request) = self.session.take_command_request() else {
            return Ok(());
        };
        let agents = self.hub.snapshot().panes;
        let queue = match &request.source {
            command::DeferredCommand::Args(args) => {
                tracing::debug!(?args, "starting attached-client command");
                command::start_resumable_command(args, &self.state, &agents, &request.context)
            }
            command::DeferredCommand::Line { line, tail } => {
                tracing::debug!(line, ?tail, "starting attached-client command line");
                command::start_resumable_command_string_with_tail(
                    line,
                    tail,
                    &self.state,
                    &agents,
                    &request.context,
                )
            }
        };
        let queue = queue.and_then(|queue| {
            self.command_runtime
                .spawn_queue(queue, Rc::clone(&self.state), COMMAND_QUEUE_BUDGET)
                .map_err(|error| command::CommandResult::err(format!("{error}\n")))
        });
        match queue {
            Ok(queued) => {
                self.phase = AttachPhase::Running(ActiveAttachCommand {
                    task: queued,
                    continuation: request.continuation,
                    allows_attach_io: false,
                });
            }
            Err(result) => {
                self.session
                    .complete_command(request.continuation, result, &self.state);
            }
        }
        Ok(())
    }

    fn drive_active_command(&mut self) -> io::Result<()> {
        let (complete, allows_attach_io) = match &mut self.phase {
            AttachPhase::Running(active) => {
                let complete = active.task.poll();
                (complete, active.task.allows_attach_io())
            }
            _ => return Ok(()),
        };
        if complete {
            let phase = std::mem::replace(&mut self.phase, AttachPhase::Session);
            let AttachPhase::Running(mut active) = phase else {
                return Ok(());
            };
            let mut result = active
                .task
                .take_output()
                .ok_or_else(|| io::Error::other("completed attach command has no result"))??;
            tracing::debug!(exit = result.exit, "attached-client command completed");
            self.background_commands
                .append(&mut result.background_commands);
            self.session
                .complete_command(active.continuation, result, &self.state);
            self.drive_session(AttachWaitReady::default())?;
        } else {
            // A queue that has not finished is waiting on something.
            let phase = std::mem::replace(&mut self.phase, AttachPhase::Finished);
            let AttachPhase::Running(mut active) = phase else {
                return Ok(());
            };
            active.allows_attach_io = allows_attach_io;
            // Each suspension parks on a completion of its own, so this wait is
            // a new source even when the same command suspends again — and a
            // burst of queued commands can finish one and suspend the next
            // without ever leaving this turn.
            self.command_generation = self.command_generation.wrapping_add(1);
            self.phase = AttachPhase::Waiting(active);
        }
        Ok(())
    }

    fn complete_pending_command(&mut self, source_ready: bool) -> io::Result<()> {
        let phase = std::mem::replace(&mut self.phase, AttachPhase::Finished);
        let mut active = match phase {
            AttachPhase::Waiting(active) => active,
            phase => {
                self.phase = phase;
                return Ok(());
            }
        };
        let _ = source_ready;
        active.task.poll();
        active.allows_attach_io = false;
        self.phase = AttachPhase::Running(active);
        self.drive_active_command()
    }

    fn apply_wait(
        &mut self,
        sources: AttachWaitSources,
        deadline: Option<Instant>,
    ) -> io::Result<()> {
        let sources = [
            (AttachRuntimeSource::Control, sources.control, 0),
            (AttachRuntimeSource::Input, sources.input, 0),
            (AttachRuntimeSource::TtyOutput, sources.tty_output, 0),
            (
                AttachRuntimeSource::Output,
                sources.output,
                sources.output_generation,
            ),
            (AttachRuntimeSource::Prompt, sources.prompt, 0),
            (AttachRuntimeSource::Render, sources.render, 0),
            (AttachRuntimeSource::Status, sources.status, 0),
            (AttachRuntimeSource::PopupRead, sources.popup_read, 0),
            (AttachRuntimeSource::PopupWrite, sources.popup_write, 0),
        ];
        let mut desired = BTreeSet::new();
        for (kind, raw_fd, logical_generation) in sources {
            if raw_fd < 0 {
                self.registrations.remove(&kind);
                continue;
            }
            let identity = (raw_fd, logical_generation);
            let source = match self.registrations.get(&kind) {
                Some((registered, source)) if *registered == identity => *source,
                _ => {
                    self.next_generation = self.next_generation.wrapping_add(1);
                    let source = EventAttachSource::Runtime {
                        source: kind,
                        generation: self.next_generation,
                    };
                    let borrowed = unsafe { BorrowedFd::borrow_raw(raw_fd) };
                    self.runtime_sources.insert(source, dup_fd(borrowed)?);
                    self.registrations.insert(kind, (identity, source));
                    source
                }
            };
            desired.insert(source);
        }
        self.runtime_desired = desired;
        self.runtime_sources
            .retain(|source, _| self.runtime_desired.contains(source));
        self.deadline = deadline;
        Ok(())
    }

    fn mark_ready(ready: &mut AttachWaitReady, source: AttachRuntimeSource) {
        match source {
            AttachRuntimeSource::Control => ready.control = true,
            AttachRuntimeSource::Input => {}
            AttachRuntimeSource::TtyOutput => ready.tty_output = true,
            AttachRuntimeSource::Output => ready.output = true,
            AttachRuntimeSource::Prompt => ready.prompt = true,
            AttachRuntimeSource::Render => ready.render = true,
            AttachRuntimeSource::Status => ready.status = true,
            AttachRuntimeSource::PopupRead => ready.popup_read = true,
            AttachRuntimeSource::PopupWrite => ready.popup_write = true,
        }
    }

    fn finish(&mut self) {
        self.phase = AttachPhase::Finished;
        self.runtime_desired.clear();
        self.runtime_sources.clear();
        self.deadline = None;
    }
}

impl ProtocolClient {
    pub(super) fn begin_attach(
        &mut self,
        target: &ActorRef<Self>,
        args: Vec<String>,
        outbox: &mut Outbox,
    ) {
        let Some((tty, context)) = self.take_identification() else {
            return;
        };
        match EventAttachClient::new(
            &args,
            tty,
            Rc::clone(&self.state),
            self.hub.clone(),
            &context,
            Rc::clone(&self.command_runtime),
        ) {
            Ok(mut attach) => {
                if let Err(error) = attach.drive(None) {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                self.protocol_state = ProtocolState::Attach(AttachClientState {
                    timer_generation: 0,
                    client: Box::new(attach),
                    timer_deadline: None,
                    input_paused: false,
                });
                self.sync_attach(target, outbox);
            }
            Err(error) => {
                self.protocol_state = ProtocolState::Command(CommandClientState {
                    operation: CommandOperation::AwaitingStep,
                });
                self.begin_response(
                    target,
                    command::CommandResult::err(error.into_message()),
                    outbox,
                );
            }
        }
    }

    pub(super) fn handle_attach_protocol_frame(
        &mut self,
        target: &ActorRef<Self>,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.protocol_state {
            ProtocolState::Attach(attach) => attach.client.handle_frame(frame),
            _ => return,
        };
        if let Err(error) = result {
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_attach(target, outbox);
    }

    pub(super) fn handle_attach_event(
        &mut self,
        target: &ActorRef<Self>,
        source: Option<EventAttachSource>,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.protocol_state {
            ProtocolState::Attach(attach) => attach.client.drive(source),
            _ => return,
        };
        if let Err(error) = result {
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_attach(target, outbox);
    }

    pub(super) fn sync_attach(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        while self.writer_is_below_high_water() {
            let frame = match &mut self.protocol_state {
                ProtocolState::Attach(attach) => attach.client.pop_frame(),
                _ => None,
            };
            let Some(frame) = frame else {
                break;
            };
            if !self.queue_frame(target, frame, outbox) {
                break;
            }
        }

        let (desired, deadline, finished, accepts_input, background_commands) =
            match &mut self.protocol_state {
                ProtocolState::Attach(attach) => (
                    attach.client.sources(),
                    attach.client.deadline(),
                    attach.client.is_finished(),
                    attach.client.accepts_protocol_input(),
                    attach.client.take_background_commands(),
                ),
                _ => return,
            };
        for request in background_commands {
            outbox.enqueue_background(self.background_commands.clone(), JobEvent::Start(request));
        }
        let was_input_paused = match &self.protocol_state {
            ProtocolState::Attach(attach) => attach.input_paused,
            _ => return,
        };
        if let ProtocolState::Attach(attach) = &mut self.protocol_state {
            attach.input_paused = !accepts_input;
        }
        let read_enabled = !self.reads_paused && accepts_input && !finished;
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, read_enabled);
        if was_input_paused && read_enabled && self.reader.has_buffered_frame() {
            self.schedule_read_continuation(target, outbox);
        }

        // A command source holds a wake rather than a token, but it is still
        // something this client has taken out and has to give back: without it
        // here, the source is never disabled and the client accumulates one
        // stale entry per suspension it goes through.
        let registered = self
            .registrations
            .attach
            .keys()
            .copied()
            .chain(
                self.registrations
                    .command_wakes
                    .iter()
                    .filter_map(|side| match side {
                        ProtocolIoSide::Attach(source) => Some(*source),
                        _ => None,
                    }),
            )
            .collect::<BTreeSet<_>>();
        for source in registered.union(&desired).copied() {
            outbox.set_protocol_interest(
                target.clone(),
                ProtocolIoSide::Attach(source),
                desired.contains(&source),
            );
        }

        let timer_changed = match &self.protocol_state {
            ProtocolState::Attach(attach) => deadline != attach.timer_deadline,
            _ => false,
        };
        if timer_changed {
            let generation = match &mut self.protocol_state {
                ProtocolState::Attach(attach) => {
                    attach.timer_deadline = deadline;
                    attach.timer_generation = attach.timer_generation.wrapping_add(1);
                    attach.timer_generation
                }
                _ => return,
            };
            match deadline {
                Some(deadline) => outbox.set_protocol_timer_event(
                    target.clone(),
                    deadline,
                    ProtocolEvent::AttachTimer(generation),
                ),
                None => outbox.cancel_protocol_timer(target.clone()),
            }
        }

        if finished {
            self.protocol_state = ProtocolState::Draining;
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
        }
        self.drive_output(target, outbox);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::message::Message;

    #[test]
    fn input_high_water_allows_one_frame_of_bounded_overshoot() {
        let mut input = AttachInput::new();
        let frame = Frame::new(Message::Resize);
        let wire_len = encode_bytes(&frame).len();

        while input.is_below_high_water() {
            input.push(Frame::new(Message::Resize));
        }

        assert!(input.bytes >= ATTACH_QUEUE_LIMIT);
        assert!(input.bytes < ATTACH_QUEUE_LIMIT + wire_len);
        assert!(input.bytes < ATTACH_QUEUE_LIMIT + MAX_IMSGSIZE);
        assert!(!input.is_below_high_water());

        let before = input.bytes;
        assert_eq!(input.pop().unwrap().msg, Message::Resize);
        assert_eq!(input.bytes, before - wire_len);
        assert!(input.is_below_high_water());
    }

    #[test]
    fn bounded_output_drains_in_protocol_order() {
        let mut output = AttachOutput::new();
        output.send(Frame::new(Message::Ready)).unwrap();
        output.send(Frame::new(Message::Exited)).unwrap();
        assert_eq!(output.pop().unwrap().msg, Message::Ready);
        assert_eq!(output.pop().unwrap().msg, Message::Exited);
    }

    #[test]
    fn tty_output_runtime_source_uses_writable_interest() {
        assert!(EventAttachClient::source_is_writable(
            EventAttachSource::Runtime {
                source: AttachRuntimeSource::TtyOutput,
                generation: 1,
            }
        ));
    }
}

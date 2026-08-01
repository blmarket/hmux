//! Event-loop ownership for interactive attach.
//!
//! The protocol actor owns the native attach session directly. Runtime frames,
//! tty readiness, pane-output notifications, and deadlines are represented as
//! bounded queues and reactor sources; no attach worker or bridge is involved.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::integration::status::StatusHub;
use crate::native::attach::{
    self, AttachCommandContinuation, AttachDrive, AttachFrameReader, AttachPrepared, AttachSession,
    AttachStartFailure, AttachWaitReady, AttachWaitSources, ClientTty,
};
use crate::server::command::{self, ClientContext};
use crate::server::pane::PaneIoMode;
use crate::server::state::ServerState;
use crate::tmux::codec::{dup_fd, encode_bytes, MAX_IMSGSIZE};
use crate::tmux::message::Frame;
use crate::tmux::traits::{FrameReader, FrameWriter};

const ATTACH_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;
const IMMEDIATE_TURN_BUDGET: usize = 64;

/// One native attach descriptor watched by the central reactor.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EventAttachSource {
    Runtime {
        source: AttachRuntimeSource,
        generation: u64,
    },
    Command,
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
pub(crate) struct AttachStartError {
    message: String,
}

impl AttachStartError {
    fn from_failure(failure: AttachStartFailure) -> Self {
        Self {
            message: failure.into_message(),
        }
    }

    pub(crate) fn into_message(self) -> String {
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

impl FrameReader for AttachInput {
    fn recv(&mut self) -> io::Result<Frame> {
        self.pop()
            .ok_or_else(|| io::Error::from(io::ErrorKind::WouldBlock))
    }
}

impl AsRawFd for AttachInput {
    fn as_raw_fd(&self) -> RawFd {
        -1
    }
}

impl AttachFrameReader for AttachInput {
    fn has_buffered_frame(&self) -> bool {
        !self.is_empty()
    }

    fn try_recv(&mut self) -> io::Result<Frame> {
        self.recv()
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

impl FrameWriter for AttachOutput {
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
pub(crate) struct EventAttachClient {
    session: AttachSession,
    state: Arc<Mutex<ServerState>>,
    hub: StatusHub,
    input: AttachInput,
    output: AttachOutput,
    runtime_sources: BTreeMap<EventAttachSource, OwnedFd>,
    runtime_desired: BTreeSet<EventAttachSource>,
    registrations: BTreeMap<AttachRuntimeSource, ((RawFd, u64), EventAttachSource)>,
    next_generation: u64,
    deadline: Option<Instant>,
    active_command: Option<ActiveAttachCommand>,
    pending_command: Option<PendingAttachCommand>,
    background_commands: Vec<command::BackgroundCommandRequest>,
    finished: bool,
}

struct ActiveAttachCommand {
    queue: command::ResumableCommandQueue,
    continuation: AttachCommandContinuation,
    allows_attach_io: bool,
}

enum PendingAttachCommand {
    Worker {
        completion: UnixStream,
        result: Arc<Mutex<Option<command::CommandSuspensionResult>>>,
    },
    PaneOutput(command::PaneOutputSuspension),
}

impl EventAttachClient {
    pub(crate) fn new(
        args: &[String],
        client_tty: ClientTty,
        state: Arc<Mutex<ServerState>>,
        hub: StatusHub,
        context: &ClientContext,
    ) -> Result<Self, AttachStartError> {
        let mut output = AttachOutput::new();
        let session = attach::start_attach_session(
            args,
            client_tty,
            &state,
            &hub,
            context,
            &mut output,
            PaneIoMode::EventLoop,
        )
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
            deadline: None,
            active_command: None,
            pending_command: None,
            background_commands: Vec::new(),
            finished: false,
        };
        attach.refresh_wait()?;
        Ok(attach)
    }

    pub(crate) fn sources(&self) -> BTreeSet<EventAttachSource> {
        if self.finished {
            BTreeSet::new()
        } else {
            self.runtime_desired.clone()
        }
    }

    pub(crate) fn source_fd(&self, source: EventAttachSource) -> Option<BorrowedFd<'_>> {
        match source {
            EventAttachSource::Command => {
                self.pending_command.as_ref().map(PendingAttachCommand::fd)
            }
            EventAttachSource::Runtime { .. } => self.runtime_sources.get(&source).map(AsFd::as_fd),
        }
    }

    pub(crate) fn source_is_writable(source: EventAttachSource) -> bool {
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

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.output.pop()
    }

    pub(crate) fn accepts_protocol_input(&self) -> bool {
        self.active_command
            .as_ref()
            .is_none_or(|command| command.allows_attach_io)
            && self.input.is_below_high_water()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished && self.output.is_empty()
    }

    pub(crate) fn take_background_commands(&mut self) -> Vec<command::BackgroundCommandRequest> {
        std::mem::take(&mut self.background_commands)
    }

    pub(crate) fn handle_frame(&mut self, frame: Frame) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.input.push(frame);
        self.refresh_wait()
    }

    pub(crate) fn drive(&mut self, source: Option<EventAttachSource>) -> io::Result<()> {
        if matches!(source, Some(EventAttachSource::Command)) {
            self.complete_pending_command()?;
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
            None | Some(EventAttachSource::Command) => AttachWaitReady::default(),
        };
        if source.is_some() {
            self.drive_session(ready)?;
        }
        self.refresh_wait()
    }

    pub(crate) fn drive_timer(&mut self) -> io::Result<()> {
        self.deadline = None;
        if self.active_command.is_some() {
            if self
                .pending_command
                .as_ref()
                .is_some_and(PendingAttachCommand::is_complete)
            {
                self.complete_pending_command()?;
            } else if self
                .active_command
                .as_ref()
                .is_some_and(|command| command.allows_attach_io)
                && self.pending_command.is_some()
            {
                self.drive_session(AttachWaitReady::default())?;
            } else if self.pending_command.is_none() {
                self.drive_active_command()?;
            }
        } else {
            self.drive_session(AttachWaitReady::default())?;
        }
        self.refresh_wait()
    }

    pub(crate) fn shutdown(&mut self) {
        self.finished = true;
        self.input.frames.clear();
        self.input.bytes = 0;
        self.output.frames.clear();
        self.output.bytes = 0;
        self.runtime_desired.clear();
        self.runtime_sources.clear();
        self.deadline = None;
        self.active_command = None;
        self.pending_command = None;
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
        if self.finished {
            return Ok(());
        }
        if self.active_command.is_none() {
            self.start_session_command()?;
        }
        if self.active_command.is_some() {
            if self.pending_command.is_none() {
                self.drive_active_command()?;
            }
            if self.active_command.is_some() {
                if self
                    .active_command
                    .as_ref()
                    .is_some_and(|command| command.allows_attach_io)
                    && self.pending_command.is_some()
                {
                    self.refresh_session_sources()?;
                    if self.finished {
                        return Ok(());
                    }
                    self.runtime_desired.insert(EventAttachSource::Command);
                    if let Some(command_deadline) = self
                        .pending_command
                        .as_ref()
                        .and_then(PendingAttachCommand::deadline)
                    {
                        self.deadline =
                            Some(self.deadline.map_or(command_deadline, |deadline| {
                                deadline.min(command_deadline)
                            }));
                    }
                    return Ok(());
                }
                self.runtime_desired.clear();
                if let Some(pending) = self.pending_command.as_ref() {
                    self.runtime_desired.insert(EventAttachSource::Command);
                    self.deadline = pending.deadline();
                } else {
                    self.deadline = Some(Instant::now());
                }
                return Ok(());
            }
        }
        self.refresh_session_sources()
    }

    fn refresh_session_sources(&mut self) -> io::Result<()> {
        for _ in 0..IMMEDIATE_TURN_BUDGET {
            match self
                .session
                .prepare_wait(&self.state, -1, !self.input.is_empty())?
            {
                AttachPrepared::Ready(ready) => self.drive_session(ready)?,
                AttachPrepared::Wait { sources, timeout } => {
                    self.apply_wait(sources, timeout)?;
                    return Ok(());
                }
                AttachPrepared::Finished => {
                    self.finish();
                    return Ok(());
                }
            }
            if self.finished {
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
        match queue {
            Ok(queue) => {
                self.active_command = Some(ActiveAttachCommand {
                    queue,
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
        let turn = self
            .active_command
            .as_mut()
            .ok_or_else(|| io::Error::other("missing active attach command"))?
            .queue
            .drive(&self.state, 64);
        match turn {
            command::ResumableCommandTurn::Pending => {
                self.deadline = Some(Instant::now());
            }
            command::ResumableCommandTurn::Suspended(suspension) => {
                let allows_attach_io = suspension.allows_attach_io();
                let pending = PendingAttachCommand::start(suspension)?;
                let complete = pending.is_complete();
                self.pending_command = Some(pending);
                if let Some(active) = self.active_command.as_mut() {
                    active.allows_attach_io = allows_attach_io;
                }
                if complete {
                    self.complete_pending_command()?;
                }
            }
            command::ResumableCommandTurn::Complete(mut result) => {
                tracing::debug!(exit = result.exit, "attached-client command completed");
                self.background_commands
                    .append(&mut result.background_commands);
                let active = self
                    .active_command
                    .take()
                    .expect("completed attach command disappeared");
                self.session
                    .complete_command(active.continuation, result, &self.state);
                self.drive_session(AttachWaitReady::default())?;
            }
        }
        Ok(())
    }

    fn complete_pending_command(&mut self) -> io::Result<()> {
        let result = self
            .pending_command
            .take()
            .ok_or_else(|| io::Error::other("attach command readiness without suspension"))?
            .take_result()?;
        let active = self
            .active_command
            .as_mut()
            .ok_or_else(|| io::Error::other("attach suspension without active command"))?;
        active.allows_attach_io = false;
        active.queue.resume(result, &self.state);
        self.drive_active_command()
    }

    fn apply_wait(&mut self, sources: AttachWaitSources, timeout: i32) -> io::Result<()> {
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
        self.deadline = (timeout >= 0).then(|| {
            Instant::now()
                .checked_add(Duration::from_millis(timeout as u64))
                .unwrap_or_else(Instant::now)
        });
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
        self.finished = true;
        self.runtime_desired.clear();
        self.runtime_sources.clear();
        self.deadline = None;
    }
}

impl PendingAttachCommand {
    fn start(suspension: command::CommandSuspension) -> io::Result<Self> {
        if let command::CommandSuspension::PaneOutput(wait) = suspension {
            return Ok(Self::PaneOutput(wait));
        }
        let (completion, mut signal) = UnixStream::pair()?;
        completion.set_nonblocking(true)?;
        let result = Arc::new(Mutex::new(None));
        let worker_result = Arc::clone(&result);
        thread::spawn(move || {
            let completed = suspension.resolve();
            if let Ok(mut result) = worker_result.lock() {
                *result = Some(completed);
            }
            let _ = signal.write_all(&[1]);
        });
        Ok(Self::Worker { completion, result })
    }

    fn fd(&self) -> BorrowedFd<'_> {
        match self {
            Self::Worker { completion, .. } => completion.as_fd(),
            Self::PaneOutput(wait) => wait.as_fd(),
        }
    }

    fn deadline(&self) -> Option<Instant> {
        match self {
            Self::Worker { .. } => None,
            Self::PaneOutput(wait) => Some(wait.deadline()),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            Self::Worker { .. } => false,
            Self::PaneOutput(wait) => wait.is_complete(),
        }
    }

    fn take_result(mut self) -> io::Result<command::CommandSuspensionResult> {
        let Self::Worker { completion, result } = &mut self else {
            let Self::PaneOutput(wait) = &mut self else {
                unreachable!();
            };
            return Ok(wait.complete());
        };
        let mut byte = [0u8; 1];
        completion.read_exact(&mut byte)?;
        result
            .lock()
            .map_err(|_| io::Error::other("attach command result poisoned"))?
            .take()
            .ok_or_else(|| io::Error::other("attach command completed without a result"))
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

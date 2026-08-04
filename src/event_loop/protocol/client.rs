//! Event-loop-owned tmux command-client and control-mode protocol state.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::os::fd::{AsFd, BorrowedFd};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::integration::status::StatusHub;
use crate::server::attach::ClientTty;
use crate::server::command::{self, ClientContext, CommandResult};
use crate::server::control::{EventControlClient, EventControlSource};
use crate::server::state::ServerState;
use crate::server::Server;
use crate::tmux::codec::{encode_bytes, ImsgReader, NonblockingImsgWriter, MAX_IMSGSIZE};
use crate::tmux::introspect::{log_frame, Direction};
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::traits::NonblockingFrameWriter;

use super::super::actor::ActorRef;
use super::super::driver::Outbox;
use super::super::job::BackgroundCommands;
use super::super::reactor::Token;
use super::super::suspend::{EventCommandRuntime, SuspensionExecutorHandle};
use super::super::timer::TimerId;
use super::attach::{EventAttachClient, EventAttachSource};
use super::command::{ActiveResumableCommand, CommandStep, CommandTransaction, CommandWork};
use super::response::CommandResponse;
use super::OUTPUT_CHUNK;

const CLIENT_CONTROL: i64 = 0x2000;
const IDENTIFY_LIMIT: usize = MAX_IMSGSIZE * 64;
const READ_FRAME_BUDGET: usize = 32;
pub(super) const FILE_STREAM: i32 = 3;
pub(super) const COMMAND_QUEUE_BUDGET: usize = 64;

/// A readiness source owned by a protocol actor.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProtocolIoSide {
    Read,
    Write,
    Command,
    Control(EventControlSource),
    Attach(EventAttachSource),
}

/// Why an event-loop protocol client stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolCloseReason {
    Completed,
    PeerClosed,
    Error(io::ErrorKind),
    IdentifyExceedsLimit,
    FrameExceedsQueueLimit,
}

/// Events delivered to a protocol actor.
pub(crate) enum ProtocolEvent {
    Start,
    Readable,
    ReadContinuation,
    Writable,
    CommandCompleted,
    CommandTimeout(u64),
    CommandStepReady(CommandStep),
    CommandQueueContinue,
    ControlReady(EventControlSource),
    ControlContinue,
    ControlTimer(u64),
    AttachReady(EventAttachSource),
    AttachTimer(u64),
}

/// Completion state retained after direct protocol descriptors are dropped.
#[derive(Clone)]
pub(crate) struct ProtocolStatus {
    close_reason: Rc<Cell<Option<ProtocolCloseReason>>>,
}

impl ProtocolStatus {
    fn new() -> Self {
        Self {
            close_reason: Rc::new(Cell::new(None)),
        }
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.close_reason.get()
    }
}

pub(super) enum ProtocolState {
    Identifying(IdentifyingState),
    Command(CommandClientState),
    Control(ControlClientState),
    Attach(AttachClientState),
    Draining,
    /// The response is fully queued and the client has been told to exit.
    ///
    /// tmux leaves the socket open here and lets the client disconnect, which
    /// is what gives a client still flushing a `save-buffer` of its own the
    /// chance to finish; closing first would discard whatever it has buffered.
    AwaitingClientClose,
    Closed,
}

pub(super) struct IdentifyingState {
    pub(super) context: ClientContext,
    pub(super) client_tty: ClientTty,
    pub(super) identify_bytes: usize,
    pub(super) control_mode: bool,
}

pub(super) struct CommandClientState {
    pub(super) operation: CommandOperation,
    pub(super) timer_generation: u64,
}

pub(super) enum CommandOperation {
    AwaitingStep,
    AwaitingQueue(ActiveResumableCommand),
    WaitingClientRead {
        transaction: CommandTransaction,
        args: Vec<String>,
        data: Vec<u8>,
    },
    WaitingClientWriteReady {
        transaction: CommandTransaction,
        args: Vec<String>,
        request: command::ClientFileWrite,
    },
    WritingClientFile {
        transaction: Option<CommandTransaction>,
        /// The originating `save-buffer`, whose `after-*` hook runs once the
        /// handshake completes.
        args: Vec<String>,
        request: command::ClientFileWrite,
        offset: usize,
        close_generated: bool,
    },
    WaitingCommand(ActiveResumableCommand),
    Responding(CommandResponse),
}

pub(super) struct ControlClientState {
    pub(super) client: Box<EventControlClient>,
    pub(super) timer_deadline: Option<Instant>,
    pub(super) timer_generation: u64,
}

pub(super) struct AttachClientState {
    pub(super) client: Box<EventAttachClient>,
    pub(super) timer_deadline: Option<Instant>,
    pub(super) timer_generation: u64,
    pub(super) input_paused: bool,
}

#[derive(Default)]
pub(super) struct ProtocolRegistrations {
    pub(super) read: Option<Token>,
    pub(super) write: Option<Token>,
    pub(super) command: Option<Token>,
    pub(super) control: BTreeMap<EventControlSource, Token>,
    pub(super) attach: BTreeMap<EventAttachSource, Token>,
    pub(super) timer: Option<TimerId>,
}

/// Event-loop-owned command, control-mode, and interactive-attach protocol state.
pub(crate) struct ProtocolClient {
    pub(super) reader: ImsgReader,
    pub(super) writer: NonblockingImsgWriter,
    pub(super) state: Arc<Mutex<ServerState>>,
    pub(super) hub: StatusHub,
    pub(super) background_commands: ActorRef<BackgroundCommands>,
    pub(super) command_runtime: Arc<dyn command::CommandRuntime>,
    pub(super) protocol_state: ProtocolState,
    pub(super) registrations: ProtocolRegistrations,
    pub(super) work_queued: BTreeSet<ProtocolIoSide>,
    pub(super) reads_paused: bool,
    pub(super) status: ProtocolStatus,
}

impl ProtocolClient {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: Server,
        background_commands: ActorRef<BackgroundCommands>,
        executor: SuspensionExecutorHandle,
    ) -> (Self, ProtocolStatus) {
        let status = ProtocolStatus::new();
        let state = server.state();
        let hub = server.status_hub();
        (
            Self {
                reader,
                writer,
                state,
                hub,
                background_commands,
                command_runtime: Arc::new(EventCommandRuntime::new(executor)),
                protocol_state: ProtocolState::Identifying(IdentifyingState {
                    context: ClientContext {
                        wait_for_interactions: true,
                        defer_attach_commands: true,
                        ..ClientContext::default()
                    },
                    client_tty: ClientTty::new(),
                    identify_bytes: 0,
                    control_mode: false,
                }),
                registrations: ProtocolRegistrations::default(),
                work_queued: BTreeSet::new(),
                reads_paused: false,
                status: status.clone(),
            },
            status,
        )
    }

    pub(crate) fn fd(&self, side: ProtocolIoSide) -> Option<BorrowedFd<'_>> {
        match side {
            ProtocolIoSide::Read => Some(self.reader.as_fd()),
            ProtocolIoSide::Write => Some(self.writer.as_fd()),
            ProtocolIoSide::Command => match &self.protocol_state {
                ProtocolState::Command(CommandClientState {
                    operation: CommandOperation::WaitingCommand(active),
                    ..
                }) => active
                    .task
                    .wait()
                    .and_then(|wait| wait.sources().first().map(|source| source.fd())),
                _ => None,
            },
            ProtocolIoSide::Control(source) => match &self.protocol_state {
                ProtocolState::Control(control) => control.client.source_fd(source),
                _ => None,
            },
            ProtocolIoSide::Attach(source) => match &self.protocol_state {
                ProtocolState::Attach(attach) => attach.client.source_fd(source),
                _ => None,
            },
        }
    }

    pub(crate) fn control_source_is_writable(source: EventControlSource) -> bool {
        EventControlClient::source_is_writable(source)
    }

    pub(crate) fn attach_source_is_writable(source: EventAttachSource) -> bool {
        EventAttachClient::source_is_writable(source)
    }

    pub(crate) fn token(&self, side: ProtocolIoSide) -> Option<Token> {
        match side {
            ProtocolIoSide::Read => self.registrations.read,
            ProtocolIoSide::Write => self.registrations.write,
            ProtocolIoSide::Command => self.registrations.command,
            ProtocolIoSide::Control(source) => self.registrations.control.get(&source).copied(),
            ProtocolIoSide::Attach(source) => self.registrations.attach.get(&source).copied(),
        }
    }

    pub(crate) fn set_token(&mut self, side: ProtocolIoSide, token: Option<Token>) {
        match side {
            ProtocolIoSide::Read => self.registrations.read = token,
            ProtocolIoSide::Write => self.registrations.write = token,
            ProtocolIoSide::Command => self.registrations.command = token,
            ProtocolIoSide::Control(source) => match token {
                Some(token) => {
                    self.registrations.control.insert(source, token);
                }
                None => {
                    self.registrations.control.remove(&source);
                }
            },
            ProtocolIoSide::Attach(source) => match token {
                Some(token) => {
                    self.registrations.attach.insert(source, token);
                }
                None => {
                    self.registrations.attach.remove(&source);
                }
            },
        }
    }

    pub(crate) fn timer(&self) -> Option<TimerId> {
        self.registrations.timer
    }

    pub(crate) fn set_timer(&mut self, timer: Option<TimerId>) {
        self.registrations.timer = timer;
    }

    pub(crate) fn mark_work_queued(&mut self, side: ProtocolIoSide) -> bool {
        if side == ProtocolIoSide::Read && (self.reads_paused || !self.accepts_protocol_input()) {
            return false;
        }
        self.work_queued.insert(side)
    }

    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        matches!(
            self.protocol_state,
            ProtocolState::Command(_)
                | ProtocolState::Control(_)
                | ProtocolState::Attach(_)
                | ProtocolState::Draining
                | ProtocolState::AwaitingClientClose
        )
    }

    #[cfg(test)]
    pub(crate) fn is_control(&self) -> bool {
        matches!(self.protocol_state, ProtocolState::Control(_))
    }

    #[cfg(test)]
    pub(crate) fn is_attach(&self) -> bool {
        matches!(self.protocol_state, ProtocolState::Attach(_))
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.status.close_reason()
    }

    fn accepts_protocol_input(&self) -> bool {
        match &self.protocol_state {
            ProtocolState::Identifying(_) | ProtocolState::Command(_) => true,
            ProtocolState::Attach(attach) => !attach.input_paused,
            // Reading on is how the departing client's close is noticed.
            ProtocolState::AwaitingClientClose => true,
            ProtocolState::Control(_) | ProtocolState::Draining | ProtocolState::Closed => false,
        }
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: ProtocolEvent,
        outbox: &mut Outbox,
    ) {
        match event {
            ProtocolEvent::Start => {
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
            }
            ProtocolEvent::Readable | ProtocolEvent::ReadContinuation => {
                self.work_queued.remove(&ProtocolIoSide::Read);
                self.handle_readable(target, outbox);
            }
            ProtocolEvent::Writable => {
                self.work_queued.remove(&ProtocolIoSide::Write);
                self.handle_writable(target, outbox);
            }
            ProtocolEvent::CommandCompleted => {
                self.work_queued.remove(&ProtocolIoSide::Command);
                self.handle_command_completed(target, true, outbox);
            }
            ProtocolEvent::CommandTimeout(generation) => {
                let current = matches!(
                    &self.protocol_state,
                    ProtocolState::Command(CommandClientState {
                        operation: CommandOperation::WaitingCommand(_),
                        timer_generation,
                    }) if generation == *timer_generation
                );
                if current {
                    self.registrations.timer = None;
                    self.handle_command_completed(target, false, outbox);
                }
            }
            ProtocolEvent::CommandStepReady(step) => {
                self.handle_command_step(target, step, outbox);
            }
            ProtocolEvent::CommandQueueContinue => {
                self.drive_resumable_command(target, outbox);
            }
            ProtocolEvent::ControlReady(source) => {
                self.work_queued.remove(&ProtocolIoSide::Control(source));
                self.handle_control_event(target, Some(source), outbox);
            }
            ProtocolEvent::ControlContinue => {
                self.handle_control_event(target, None, outbox);
            }
            ProtocolEvent::ControlTimer(generation) => {
                let current = matches!(
                    &self.protocol_state,
                    ProtocolState::Control(control) if generation == control.timer_generation
                );
                if current {
                    self.registrations.timer = None;
                    if let ProtocolState::Control(control) = &mut self.protocol_state {
                        control.timer_deadline = None;
                    }
                    self.handle_control_event(target, None, outbox);
                }
            }
            ProtocolEvent::AttachReady(source) => {
                self.work_queued.remove(&ProtocolIoSide::Attach(source));
                self.handle_attach_event(target, Some(source), outbox);
            }
            ProtocolEvent::AttachTimer(generation) => {
                let current = matches!(
                    &self.protocol_state,
                    ProtocolState::Attach(attach) if generation == attach.timer_generation
                );
                if current {
                    self.registrations.timer = None;
                    let result = match &mut self.protocol_state {
                        ProtocolState::Attach(attach) => {
                            attach.timer_deadline = None;
                            attach.client.drive_timer()
                        }
                        _ => return,
                    };
                    if let Err(error) = result {
                        self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                        return;
                    }
                    self.sync_attach(target, outbox);
                }
            }
        }
    }

    fn handle_readable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused || !self.accepts_protocol_input() {
            return;
        }

        for _ in 0..READ_FRAME_BUDGET {
            let frame = match self.reader.try_recv() {
                Ok(frame) => frame,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    self.close(target, ProtocolCloseReason::PeerClosed, outbox);
                    return;
                }
                Err(error) => {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
            };
            log_frame(Direction::ClientToServer, &frame);

            if frame.version != PROTOCOL_VERSION {
                self.protocol_state = ProtocolState::Draining;
                self.queue_frame(target, Frame::new(Message::Version), outbox);
                self.drive_output(target, outbox);
                return;
            }

            match self.protocol_state {
                ProtocolState::Identifying(_) => self.handle_identifying(target, frame, outbox),
                ProtocolState::Command(_) => self.handle_direct_frame(target, frame, outbox),
                ProtocolState::Control(_) => {
                    self.handle_control_protocol_frame(target, frame, outbox)
                }
                ProtocolState::Attach(_) => {
                    self.handle_attach_protocol_frame(target, frame, outbox)
                }
                // Whatever a client says after being told to exit is moot; the
                // read is only here to see the socket close.
                ProtocolState::AwaitingClientClose => {}
                ProtocolState::Draining | ProtocolState::Closed => return,
            }
            if self.reads_paused || !self.accepts_protocol_input() {
                return;
            }
        }

        self.schedule_read_continuation(target, outbox);
    }

    fn handle_identifying(
        &mut self,
        target: &ActorRef<Self>,
        mut frame: Frame,
        outbox: &mut Outbox,
    ) {
        if Self::is_identify_message(&frame.msg) {
            let frame_bytes = encode_bytes(&frame).len();
            let Some(identifying) = self.identifying_mut() else {
                return;
            };
            if frame_bytes > IDENTIFY_LIMIT.saturating_sub(identifying.identify_bytes) {
                self.close(target, ProtocolCloseReason::IdentifyExceedsLimit, outbox);
                return;
            }
            identifying.identify_bytes += frame_bytes;
        }
        self.observe_identify(&mut frame);
        if let Message::Command(args) = &mut frame.msg {
            if args.is_empty() {
                *args = self.default_client_command();
            }
        }
        match &frame.msg {
            Message::Command(args)
                if self
                    .identifying()
                    .is_some_and(|identifying| identifying.control_mode) =>
            {
                let args = args.clone();
                self.begin_control(target, args, outbox);
            }
            Message::Command(args)
                if self
                    .identifying()
                    .is_some_and(|identifying| !identifying.control_mode)
                    && command::classify(args) == command::Intent::Command =>
            {
                let args = args.clone();
                self.begin_command(target, args, outbox);
            }
            Message::Command(args) => {
                let args = args.clone();
                self.begin_attach(target, args, outbox);
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown => {
                self.close(target, ProtocolCloseReason::Completed, outbox);
            }
            _ => {}
        }
    }

    /// The command line an argument-less client runs, from the server option.
    /// Falls back to tmux's default when the state lock is unavailable.
    fn default_client_command(&self) -> Vec<String> {
        self.state
            .lock()
            .map(|state| command::default_client_command(&state))
            .unwrap_or_else(|_| vec!["new-session".to_string()])
    }

    fn is_identify_message(message: &Message) -> bool {
        matches!(
            message,
            Message::IdentifyFlags(_)
                | Message::IdentifyLongFlags(_)
                | Message::Flags(_)
                | Message::IdentifyTerminfo(_)
                | Message::IdentifyFeatures(_)
                | Message::IdentifyTerm(_)
                | Message::IdentifyTtyName(_)
                | Message::IdentifyClientPid(_)
                | Message::IdentifyCwd(_)
                | Message::IdentifyEnviron(_)
                | Message::IdentifyStdin
                | Message::IdentifyStdout
                | Message::IdentifyDone
        )
    }

    pub(super) fn identifying(&self) -> Option<&IdentifyingState> {
        match &self.protocol_state {
            ProtocolState::Identifying(identifying) => Some(identifying),
            _ => None,
        }
    }

    pub(super) fn identifying_mut(&mut self) -> Option<&mut IdentifyingState> {
        match &mut self.protocol_state {
            ProtocolState::Identifying(identifying) => Some(identifying),
            _ => None,
        }
    }

    pub(super) fn take_identification(&mut self) -> Option<(ClientTty, ClientContext)> {
        let identifying = self.identifying_mut()?;
        identifying.identify_bytes = 0;
        Some((
            std::mem::replace(&mut identifying.client_tty, ClientTty::new()),
            identifying.context.clone(),
        ))
    }

    fn observe_identify(&mut self, frame: &mut Frame) {
        let Some(identifying) = self.identifying_mut() else {
            return;
        };
        match &frame.msg {
            Message::IdentifyFlags(flags) => {
                identifying.control_mode |= i64::from(*flags) & CLIENT_CONTROL != 0;
                identifying.client_tty.flags |= i64::from(*flags);
            }
            Message::IdentifyLongFlags(flags) | Message::Flags(flags) => {
                identifying.control_mode |= *flags & CLIENT_CONTROL != 0;
                identifying.client_tty.flags |= *flags;
            }
            Message::IdentifyTerminfo(capability) => {
                identifying.client_tty.terminfo.push(capability.clone());
            }
            Message::IdentifyFeatures(features) => {
                identifying.client_tty.features |= *features as u32;
            }
            Message::IdentifyTerm(term) => {
                identifying.client_tty.term = Some(term.clone());
            }
            Message::IdentifyTtyName(tty_name) => {
                identifying.context.tty_name = Some(tty_name.clone());
                identifying.client_tty.tty_name = Some(tty_name.clone());
            }
            Message::IdentifyClientPid(pid) => {
                identifying.context.client_pid = Some(*pid);
                identifying.client_tty.client_pid = Some(*pid);
            }
            Message::IdentifyCwd(cwd) => {
                identifying.context.cwd = Some(cwd.into());
            }
            Message::IdentifyEnviron(entry) => {
                identifying.context.environment.push(entry.clone());
            }
            Message::IdentifyStdin => {
                identifying.client_tty.stdin = frame.fd.take();
            }
            Message::IdentifyStdout => {
                identifying.client_tty.stdout = frame.fd.take();
            }
            _ => {}
        }
    }

    pub(super) fn begin_response(
        &mut self,
        target: &ActorRef<Self>,
        result: CommandResult,
        outbox: &mut Outbox,
    ) {
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        command.operation = CommandOperation::Responding(CommandResponse::new(result));
        self.drive_output(target, outbox);
    }

    fn next_generated_frame(&mut self) -> GeneratedFrame {
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return GeneratedFrame::Blocked;
        };
        match &mut command.operation {
            CommandOperation::WritingClientFile {
                transaction,
                args,
                request,
                offset,
                close_generated,
            } => {
                if *offset < request.data.len() {
                    let end = (*offset + OUTPUT_CHUNK).min(request.data.len());
                    let frame = Frame::new(Message::Write {
                        stream: FILE_STREAM,
                        data: request.data[*offset..end].to_vec(),
                    });
                    *offset = end;
                    GeneratedFrame::Frame(frame)
                } else if !*close_generated {
                    *close_generated = true;
                    GeneratedFrame::Frame(Frame::new(Message::WriteClose {
                        stream: FILE_STREAM,
                    }))
                } else {
                    GeneratedFrame::ClientFileComplete(
                        transaction
                            .take()
                            .expect("client file transaction was already taken"),
                        std::mem::take(args),
                    )
                }
            }
            CommandOperation::Responding(response) => match response.next_frame() {
                Some(frame) => GeneratedFrame::Frame(frame),
                None if response.is_complete() => GeneratedFrame::ResponseComplete,
                None => GeneratedFrame::Blocked,
            },
            _ => GeneratedFrame::Blocked,
        }
    }

    pub(super) fn drive_output(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused && self.writer_is_below_high_water() {
            self.reads_paused = false;
            if self.accepts_protocol_input() {
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
                if self.reader.has_buffered_frame() {
                    self.schedule_read_continuation(target, outbox);
                }
            }
        }

        while self.writer_is_below_high_water() {
            match self.next_generated_frame() {
                GeneratedFrame::Frame(frame) => {
                    if !self.queue_frame(target, frame, outbox) {
                        return;
                    }
                }
                GeneratedFrame::ClientFileComplete(mut transaction, args) => {
                    let result = CommandResult::ok("");
                    // The write is the command; its `after-*` hook belongs to
                    // the completed handshake, not to the queue step that never
                    // ran it.
                    if let Ok(mut state) = self.state.lock() {
                        command::run_client_file_after_hook(
                            &args,
                            &mut state,
                            &transaction.context,
                        );
                    }
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    self.start_command_work(target, CommandWork::Advance(transaction), outbox);
                }
                GeneratedFrame::ResponseComplete => {
                    self.protocol_state = ProtocolState::AwaitingClientClose;
                    outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
                    break;
                }
                GeneratedFrame::Blocked => break,
            }
        }

        if !self.writer_is_below_high_water() {
            self.reads_paused = true;
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
        }
        let pending = self.writer.has_pending();
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, pending);
        if matches!(self.protocol_state, ProtocolState::Draining) && !pending {
            self.close(target, ProtocolCloseReason::Completed, outbox);
        }
    }

    pub(super) fn writer_is_below_high_water(&self) -> bool {
        self.writer.is_below_high_water()
    }

    pub(super) fn queue_frame(
        &mut self,
        target: &ActorRef<Self>,
        frame: Frame,
        outbox: &mut Outbox,
    ) -> bool {
        log_frame(Direction::ServerToClient, &frame);
        match self.writer.try_queue(frame) {
            Ok(()) => {
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, true);
                if !self.writer_is_below_high_water() {
                    self.reads_paused = true;
                    outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
                }
                true
            }
            Err(_) => {
                self.close(target, ProtocolCloseReason::FrameExceedsQueueLimit, outbox);
                false
            }
        }
    }

    fn handle_writable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        match self.writer.try_flush() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                return;
            }
        }
        self.drive_output(target, outbox);
        if self.status.close_reason().is_none() {
            match self.protocol_state {
                ProtocolState::Control(_) => self.sync_control(target, outbox),
                ProtocolState::Attach(_) => self.sync_attach(target, outbox),
                _ => {}
            }
        }
    }

    pub(super) fn schedule_read_continuation(
        &mut self,
        target: &ActorRef<Self>,
        outbox: &mut Outbox,
    ) {
        if self.mark_work_queued(ProtocolIoSide::Read) {
            outbox.enqueue_protocol(target.clone(), ProtocolEvent::ReadContinuation);
        }
    }

    pub(super) fn close(
        &mut self,
        target: &ActorRef<Self>,
        reason: ProtocolCloseReason,
        outbox: &mut Outbox,
    ) {
        self.status.close_reason.set(Some(reason));
        for side in [
            ProtocolIoSide::Read,
            ProtocolIoSide::Write,
            ProtocolIoSide::Command,
        ] {
            outbox.set_protocol_interest(target.clone(), side, false);
        }
        for source in self
            .registrations
            .control
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Control(source), false);
        }
        for source in self
            .registrations
            .attach
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Attach(source), false);
        }
        if let ProtocolState::Attach(attach) = &mut self.protocol_state {
            attach.client.shutdown();
        }
        self.protocol_state = ProtocolState::Closed;
        outbox.cancel_protocol_timer(target.clone());
        outbox.stop_protocol(target.clone());
    }
}

enum GeneratedFrame {
    Frame(Frame),
    ClientFileComplete(CommandTransaction, Vec<String>),
    ResponseComplete,
    Blocked,
}

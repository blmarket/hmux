//! Event-loop-owned tmux command-client and control-mode protocol state.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::integration::status::{StatusHub, StatusSnapshot, StatusSubscription};
use crate::tmux::codec::{encode_bytes, ImsgReader, NonblockingImsgWriter, MAX_IMSGSIZE};
use crate::tmux::introspect::{log_frame, Direction};
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::native::attach::ClientTty;
use crate::tmux::native::command::{self, ClientContext, CommandResult};
use crate::tmux::native::protocol::{EventControlClient, EventControlSource};
use crate::tmux::native::state::ServerState;
use crate::tmux::native::NativeServer;
use crate::tmux::traits::NonblockingFrameWriter;

use super::actor::ActorRef;
use super::attach::{EventAttachClient, EventAttachSource};
use super::client::READ_FRAME_BUDGET;
use super::driver::{Outbox, PairingHandle};
use super::job::BackgroundCommands;
use super::pairing::PairingCloseReason;
use super::reactor::Token;
use super::timer::TimerId;

const CLIENT_CONTROL: i64 = 0x2000;
const FALLBACK_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;
const FILE_STREAM: i32 = 3;
const OUTPUT_CHUNK: usize = 8 * 1024;
const COMMAND_QUEUE_BUDGET: usize = 64;
#[cfg(not(test))]
const STATUS_HEARTBEAT: Duration = Duration::from_secs(30);
#[cfg(test)]
const STATUS_HEARTBEAT: Duration = Duration::from_millis(20);

/// A readiness source owned by a protocol actor.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProtocolIoSide {
    Read,
    Write,
    Command,
    Status,
    Control(EventControlSource),
    Attach(EventAttachSource),
}

/// Why an event-loop protocol client stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolCloseReason {
    Completed,
    PeerClosed,
    Error(io::ErrorKind),
    Shutdown,
    PreludeExceedsQueueLimit,
    FrameExceedsQueueLimit,
    Fallback(PairingCloseReason),
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
    StatusReady,
    StatusHeartbeat(u64),
    ControlReady(EventControlSource),
    ControlContinue,
    ControlTimer(u64),
    AttachReady(EventAttachSource),
    AttachTimer(u64),
    Shutdown,
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

enum ProtocolMode {
    Identifying,
    Direct,
    Control(Box<EventControlClient>),
    Attach(Box<EventAttachClient>),
    HandingOff,
    Fallback(PairingHandle),
}

enum DirectOperation {
    Idle,
    WaitingClientRead {
        transaction: CommandTransaction,
        args: Vec<String>,
        data: Vec<u8>,
    },
    WaitingClientWriteReady {
        transaction: CommandTransaction,
        request: command::ClientFileWrite,
    },
    WritingClientFile {
        transaction: Option<CommandTransaction>,
        request: command::ClientFileWrite,
        offset: usize,
        close_generated: bool,
    },
    WaitingCommand {
        pending: bool,
    },
    WaitingStatus {
        since: u64,
        generation: u64,
    },
    Responding(CommandResponse),
}

/// Direct command/control state plus an optional attach compatibility handoff.
pub(crate) struct ProtocolClient {
    reader: Option<ImsgReader>,
    writer: Option<NonblockingImsgWriter>,
    server: NativeServer,
    state: Arc<Mutex<ServerState>>,
    hub: StatusHub,
    background_commands: ActorRef<BackgroundCommands>,
    mode: ProtocolMode,
    context: ClientContext,
    control_mode: bool,
    prelude: VecDeque<Frame>,
    prelude_bytes: usize,
    operation: DirectOperation,
    retry: Option<Frame>,
    completion: Option<PendingCommand>,
    resumable_command: Option<ActiveResumableCommand>,
    status_subscription: Option<StatusSubscription>,
    read_token: Option<Token>,
    write_token: Option<Token>,
    command_token: Option<Token>,
    status_token: Option<Token>,
    control_tokens: BTreeMap<EventControlSource, Token>,
    attach_tokens: BTreeMap<EventAttachSource, Token>,
    status_timer: Option<TimerId>,
    status_generation: u64,
    command_generation: u64,
    control_timer_deadline: Option<Instant>,
    control_timer_generation: u64,
    attach_timer_deadline: Option<Instant>,
    attach_timer_generation: u64,
    read_work_queued: bool,
    write_work_queued: bool,
    command_work_queued: bool,
    status_work_queued: bool,
    control_work_queued: BTreeSet<EventControlSource>,
    attach_work_queued: BTreeSet<EventAttachSource>,
    reads_paused: bool,
    attach_input_paused: bool,
    close_after_flush: bool,
    status: ProtocolStatus,
}

impl ProtocolClient {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: NativeServer,
        background_commands: ActorRef<BackgroundCommands>,
    ) -> (Self, ProtocolStatus) {
        let status = ProtocolStatus::new();
        let state = server.state();
        let hub = server.status_hub();
        (
            Self {
                reader: Some(reader),
                writer: Some(writer),
                server,
                state,
                hub,
                background_commands,
                mode: ProtocolMode::Identifying,
                context: ClientContext {
                    wait_for_interactions: true,
                    defer_background_commands: true,
                    defer_attach_commands: true,
                    ..ClientContext::default()
                },
                control_mode: false,
                prelude: VecDeque::new(),
                prelude_bytes: 0,
                operation: DirectOperation::Idle,
                retry: None,
                completion: None,
                resumable_command: None,
                status_subscription: None,
                read_token: None,
                write_token: None,
                command_token: None,
                status_token: None,
                control_tokens: BTreeMap::new(),
                attach_tokens: BTreeMap::new(),
                status_timer: None,
                status_generation: 0,
                command_generation: 0,
                control_timer_deadline: None,
                control_timer_generation: 0,
                attach_timer_deadline: None,
                attach_timer_generation: 0,
                read_work_queued: false,
                write_work_queued: false,
                command_work_queued: false,
                status_work_queued: false,
                control_work_queued: BTreeSet::new(),
                attach_work_queued: BTreeSet::new(),
                reads_paused: false,
                attach_input_paused: false,
                close_after_flush: false,
                status: status.clone(),
            },
            status,
        )
    }

    pub(crate) fn fd(&self, side: ProtocolIoSide) -> Option<BorrowedFd<'_>> {
        match side {
            ProtocolIoSide::Read => self.reader.as_ref().map(AsFd::as_fd),
            ProtocolIoSide::Write => self.writer.as_ref().map(AsFd::as_fd),
            ProtocolIoSide::Command => self.completion.as_ref().map(PendingCommand::fd),
            ProtocolIoSide::Status => self
                .status_subscription
                .as_ref()
                .map(StatusSubscription::as_fd),
            ProtocolIoSide::Control(source) => match &self.mode {
                ProtocolMode::Control(control) => control.source_fd(source),
                _ => None,
            },
            ProtocolIoSide::Attach(source) => match &self.mode {
                ProtocolMode::Attach(attach) => attach.source_fd(source),
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
            ProtocolIoSide::Read => self.read_token,
            ProtocolIoSide::Write => self.write_token,
            ProtocolIoSide::Command => self.command_token,
            ProtocolIoSide::Status => self.status_token,
            ProtocolIoSide::Control(source) => self.control_tokens.get(&source).copied(),
            ProtocolIoSide::Attach(source) => self.attach_tokens.get(&source).copied(),
        }
    }

    pub(crate) fn set_token(&mut self, side: ProtocolIoSide, token: Option<Token>) {
        match side {
            ProtocolIoSide::Read => self.read_token = token,
            ProtocolIoSide::Write => self.write_token = token,
            ProtocolIoSide::Command => {
                self.command_token = token;
                if token.is_none() {
                    self.completion = None;
                }
            }
            ProtocolIoSide::Status => {
                self.status_token = token;
                if token.is_none() {
                    self.status_subscription = None;
                }
            }
            ProtocolIoSide::Control(source) => match token {
                Some(token) => {
                    self.control_tokens.insert(source, token);
                }
                None => {
                    self.control_tokens.remove(&source);
                }
            },
            ProtocolIoSide::Attach(source) => match token {
                Some(token) => {
                    self.attach_tokens.insert(source, token);
                }
                None => {
                    self.attach_tokens.remove(&source);
                }
            },
        }
    }

    pub(crate) fn status_timer(&self) -> Option<TimerId> {
        self.status_timer
    }

    pub(crate) fn set_status_timer(&mut self, timer: Option<TimerId>) {
        self.status_timer = timer;
    }

    pub(crate) fn mark_work_queued(&mut self, side: ProtocolIoSide) -> bool {
        if matches!(
            self.mode,
            ProtocolMode::HandingOff | ProtocolMode::Fallback(_)
        ) {
            return false;
        }
        let queued = match side {
            ProtocolIoSide::Read => &mut self.read_work_queued,
            ProtocolIoSide::Write => &mut self.write_work_queued,
            ProtocolIoSide::Command => &mut self.command_work_queued,
            ProtocolIoSide::Status => &mut self.status_work_queued,
            ProtocolIoSide::Control(source) => {
                return self.control_work_queued.insert(source);
            }
            ProtocolIoSide::Attach(source) => {
                return self.attach_work_queued.insert(source);
            }
        };
        if *queued
            || (side == ProtocolIoSide::Read
                && (self.reads_paused
                    || self.attach_input_paused
                    || matches!(self.operation, DirectOperation::WaitingStatus { .. })))
        {
            return false;
        }
        *queued = true;
        true
    }

    pub(crate) fn install_fallback(&mut self, pairing: PairingHandle) {
        self.mode = ProtocolMode::Fallback(pairing);
    }

    pub(crate) fn is_active(&self) -> bool {
        match &self.mode {
            ProtocolMode::Fallback(pairing) => pairing.is_alive(),
            ProtocolMode::Identifying
            | ProtocolMode::Direct
            | ProtocolMode::Control(_)
            | ProtocolMode::Attach(_)
            | ProtocolMode::HandingOff => true,
        }
    }

    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        matches!(
            self.mode,
            ProtocolMode::Direct | ProtocolMode::Control(_) | ProtocolMode::Attach(_)
        )
    }

    #[cfg(test)]
    pub(crate) fn is_control(&self) -> bool {
        matches!(self.mode, ProtocolMode::Control(_))
    }

    #[cfg(test)]
    pub(crate) fn is_attach(&self) -> bool {
        matches!(self.mode, ProtocolMode::Attach(_))
    }

    #[cfg(test)]
    pub(crate) fn is_fallback(&self) -> bool {
        matches!(self.mode, ProtocolMode::Fallback(_))
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        match &self.mode {
            ProtocolMode::Fallback(pairing) => pairing
                .status()
                .close_reason()
                .map(ProtocolCloseReason::Fallback),
            _ => self.status.close_reason(),
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
                self.read_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ProtocolEvent::Writable => {
                self.write_work_queued = false;
                self.handle_writable(target, outbox);
            }
            ProtocolEvent::CommandCompleted => {
                self.command_work_queued = false;
                self.handle_command_completed(target, outbox);
            }
            ProtocolEvent::CommandTimeout(generation) => {
                if generation == self.command_generation {
                    self.status_timer = None;
                    self.handle_command_completed(target, outbox);
                }
            }
            ProtocolEvent::CommandStepReady(step) => {
                self.handle_command_step(target, step, outbox);
            }
            ProtocolEvent::CommandQueueContinue => {
                self.drive_resumable_command(target, outbox);
            }
            ProtocolEvent::StatusReady => {
                self.status_work_queued = false;
                self.handle_status_ready(target, outbox);
            }
            ProtocolEvent::StatusHeartbeat(generation) => {
                self.handle_status_heartbeat(target, generation, outbox);
            }
            ProtocolEvent::ControlReady(source) => {
                self.control_work_queued.remove(&source);
                self.handle_control_event(target, Some(source), outbox);
            }
            ProtocolEvent::ControlContinue => {
                self.handle_control_event(target, None, outbox);
            }
            ProtocolEvent::ControlTimer(generation) => {
                if generation == self.control_timer_generation {
                    self.status_timer = None;
                    self.control_timer_deadline = None;
                    self.handle_control_event(target, None, outbox);
                }
            }
            ProtocolEvent::AttachReady(source) => {
                self.attach_work_queued.remove(&source);
                self.handle_attach_event(target, Some(source), outbox);
            }
            ProtocolEvent::AttachTimer(generation) => {
                if generation == self.attach_timer_generation {
                    self.status_timer = None;
                    self.attach_timer_deadline = None;
                    let result = match &mut self.mode {
                        ProtocolMode::Attach(attach) => attach.drive_timer(),
                        _ => return,
                    };
                    if let Err(error) = result {
                        self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                        return;
                    }
                    self.sync_attach(target, outbox);
                }
            }
            ProtocolEvent::Shutdown => {
                self.close(target, ProtocolCloseReason::Shutdown, outbox);
            }
        }
    }

    fn handle_readable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused || self.reader.is_none() {
            return;
        }

        for _ in 0..READ_FRAME_BUDGET {
            let frame = match self
                .reader
                .as_mut()
                .expect("protocol reader disappeared")
                .try_recv()
            {
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
                self.prelude.clear();
                self.operation = DirectOperation::Idle;
                self.close_after_flush = true;
                self.queue_frame(target, Frame::new(Message::Version), outbox);
                self.drive_output(target, outbox);
                return;
            }

            match self.mode {
                ProtocolMode::Identifying => self.handle_identifying(target, frame, outbox),
                ProtocolMode::Direct => self.handle_direct_frame(target, frame, outbox),
                ProtocolMode::Control(_) => {
                    self.handle_control_protocol_frame(target, frame, outbox)
                }
                ProtocolMode::Attach(_) => self.handle_attach_protocol_frame(target, frame, outbox),
                ProtocolMode::HandingOff | ProtocolMode::Fallback(_) => return,
            }
            if self.reader.is_none()
                || self.retry.is_some()
                || self.attach_input_paused
                || self.close_after_flush
                || matches!(self.operation, DirectOperation::WaitingStatus { .. })
            {
                return;
            }
        }

        self.schedule_read_continuation(target, outbox);
    }

    fn handle_identifying(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) {
        self.observe_identify(&frame);
        match &frame.msg {
            Message::Command(args) if self.control_mode => {
                let args = args.clone();
                self.begin_control(target, args, outbox);
            }
            Message::Command(args)
                if !self.control_mode && command::classify(args) == command::Intent::Command =>
            {
                let args = args.clone();
                self.prelude.clear();
                self.prelude_bytes = 0;
                self.mode = ProtocolMode::Direct;
                self.begin_command(target, args, outbox);
            }
            Message::Command(args) if !self.control_mode => {
                let args = args.clone();
                self.begin_attach(target, args, outbox);
            }
            Message::StatusWait { since } if !self.control_mode => {
                self.prelude.clear();
                self.prelude_bytes = 0;
                self.mode = ProtocolMode::Direct;
                self.begin_status_wait(target, *since, outbox);
            }
            Message::Command(_) | Message::StatusWait { .. } => {
                if self.push_prelude(frame).is_err() {
                    self.close(
                        target,
                        ProtocolCloseReason::PreludeExceedsQueueLimit,
                        outbox,
                    );
                    return;
                }
                self.begin_fallback(target, outbox);
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown => {
                self.close(target, ProtocolCloseReason::Completed, outbox);
            }
            _ => {
                if self.push_prelude(frame).is_err() {
                    self.close(
                        target,
                        ProtocolCloseReason::PreludeExceedsQueueLimit,
                        outbox,
                    );
                }
            }
        }
    }

    fn observe_identify(&mut self, frame: &Frame) {
        match &frame.msg {
            Message::IdentifyFlags(flags) => {
                self.control_mode |= i64::from(*flags) & CLIENT_CONTROL != 0;
            }
            Message::IdentifyLongFlags(flags) | Message::Flags(flags) => {
                self.control_mode |= *flags & CLIENT_CONTROL != 0;
            }
            Message::IdentifyTtyName(tty_name) => {
                self.context.tty_name = Some(tty_name.clone());
            }
            Message::IdentifyClientPid(pid) => {
                self.context.client_pid = Some(*pid);
            }
            Message::IdentifyCwd(cwd) => {
                self.context.cwd = Some(cwd.into());
            }
            Message::IdentifyEnviron(entry) => {
                self.context.environment.push(entry.clone());
            }
            _ => {}
        }
    }

    fn push_prelude(&mut self, frame: Frame) -> Result<(), Frame> {
        let frame_bytes = encode_bytes(&frame).len();
        if frame_bytes > FALLBACK_QUEUE_LIMIT.saturating_sub(self.prelude_bytes) {
            return Err(frame);
        }
        self.prelude_bytes += frame_bytes;
        self.prelude.push_back(frame);
        Ok(())
    }

    fn begin_control(&mut self, target: &ActorRef<Self>, args: Vec<String>, outbox: &mut Outbox) {
        let tty = self.take_client_tty();
        match EventControlClient::new(
            &args,
            tty,
            Arc::clone(&self.state),
            self.hub.clone(),
            &self.context,
        ) {
            Ok(mut control) => {
                if let Err(error) = control.drive(None) {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                self.mode = ProtocolMode::Control(Box::new(control));
                self.operation = DirectOperation::Idle;
                // Control mode moves subsequent input to the passed stdin fd.
                // The native control loop likewise stops reading imsg here.
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
                self.sync_control(target, outbox);
            }
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            }
        }
    }

    fn begin_attach(&mut self, target: &ActorRef<Self>, args: Vec<String>, outbox: &mut Outbox) {
        let tty = self.take_client_tty();
        match EventAttachClient::new(
            &args,
            tty,
            Arc::clone(&self.state),
            self.hub.clone(),
            &self.context,
        ) {
            Ok(mut attach) => {
                if let Err(error) = attach.drive(None) {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                self.mode = ProtocolMode::Attach(Box::new(attach));
                self.operation = DirectOperation::Idle;
                self.sync_attach(target, outbox);
            }
            Err(error) => {
                self.mode = ProtocolMode::Direct;
                self.begin_response(target, CommandResult::err(error.into_message()), outbox);
            }
        }
    }

    fn take_client_tty(&mut self) -> ClientTty {
        let mut tty = ClientTty::new();
        while let Some(frame) = self.prelude.pop_front() {
            match frame.msg {
                Message::IdentifyFlags(flags) => tty.flags |= i64::from(flags),
                Message::IdentifyLongFlags(flags) | Message::Flags(flags) => tty.flags |= flags,
                Message::IdentifyTerminfo(capability) => tty.terminfo.push(capability),
                Message::IdentifyFeatures(features) => tty.features |= features as u32,
                Message::IdentifyTerm(term) => tty.term = Some(term),
                Message::IdentifyTtyName(name) => tty.tty_name = Some(name),
                Message::IdentifyClientPid(pid) => tty.client_pid = Some(pid),
                Message::IdentifyStdin => tty.stdin = frame.fd,
                Message::IdentifyStdout => tty.stdout = frame.fd,
                _ => {}
            }
        }
        self.prelude_bytes = 0;
        tty
    }

    fn handle_control_protocol_frame(
        &mut self,
        target: &ActorRef<Self>,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        if matches!(
            frame.msg,
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown
        ) {
            self.close(target, ProtocolCloseReason::Completed, outbox);
        }
    }

    fn handle_control_event(
        &mut self,
        target: &ActorRef<Self>,
        source: Option<EventControlSource>,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.mode {
            ProtocolMode::Control(control) => control.drive(source),
            _ => return,
        };
        if let Err(error) = result {
            tracing::warn!(%error, "event-loop control client failed");
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_control(target, outbox);
    }

    fn sync_control(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let (desired, deadline, finished) = match &self.mode {
            ProtocolMode::Control(control) => (
                control.sources().into_iter().collect::<BTreeSet<_>>(),
                control.deadline(Instant::now()),
                control.is_finished(),
            ),
            _ => return,
        };
        let registered = self.control_tokens.keys().copied().collect::<BTreeSet<_>>();
        for source in registered.union(&desired).copied() {
            outbox.set_protocol_interest(
                target.clone(),
                ProtocolIoSide::Control(source),
                desired.contains(&source),
            );
        }

        if deadline != self.control_timer_deadline {
            self.control_timer_deadline = deadline;
            self.control_timer_generation = self.control_timer_generation.wrapping_add(1);
            match deadline {
                Some(deadline) => outbox.set_protocol_timer_event(
                    target.clone(),
                    deadline,
                    ProtocolEvent::ControlTimer(self.control_timer_generation),
                ),
                None => outbox.cancel_protocol_timer(target.clone()),
            }
        }

        loop {
            let frame = match &mut self.mode {
                ProtocolMode::Control(control) => control.pop_frame(),
                _ => None,
            };
            let Some(frame) = frame else {
                break;
            };
            if !self.queue_frame(target, frame, outbox) {
                return;
            }
        }
        let continue_input = match &mut self.mode {
            ProtocolMode::Control(control) => control.take_input_continuation(),
            _ => false,
        };
        if continue_input && self.control_work_queued.insert(EventControlSource::Input) {
            outbox.enqueue_protocol(
                target.clone(),
                ProtocolEvent::ControlReady(EventControlSource::Input),
            );
        }
        let (continue_command, background_commands) = match &mut self.mode {
            ProtocolMode::Control(control) => (
                control.take_command_continuation(),
                control.take_background_commands(),
            ),
            _ => (false, Vec::new()),
        };
        if continue_command {
            outbox.enqueue_protocol(target.clone(), ProtocolEvent::ControlContinue);
        }
        for request in background_commands {
            outbox.enqueue_background(
                self.background_commands.clone(),
                super::job::JobEvent::Start(request),
            );
        }
        if finished {
            self.close_after_flush = true;
        }
        self.drive_output(target, outbox);
    }

    fn handle_attach_protocol_frame(
        &mut self,
        target: &ActorRef<Self>,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.mode {
            ProtocolMode::Attach(attach) => attach.handle_frame(frame),
            _ => return,
        };
        if let Err(error) = result {
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_attach(target, outbox);
    }

    fn handle_attach_event(
        &mut self,
        target: &ActorRef<Self>,
        source: Option<EventAttachSource>,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.mode {
            ProtocolMode::Attach(attach) => attach.drive(source),
            _ => return,
        };
        if let Err(error) = result {
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_attach(target, outbox);
    }

    fn sync_attach(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        loop {
            let frame = match &mut self.mode {
                ProtocolMode::Attach(attach) => attach.pop_frame(),
                _ => None,
            };
            let Some(frame) = frame else {
                break;
            };
            if !self.queue_frame(target, frame, outbox) {
                break;
            }
        }

        let (desired, deadline, finished, accepts_input, background_commands) = match &mut self.mode
        {
            ProtocolMode::Attach(attach) => (
                attach.sources(),
                attach.deadline(),
                attach.is_finished(),
                attach.accepts_protocol_input(),
                attach.take_background_commands(),
            ),
            _ => return,
        };
        for request in background_commands {
            outbox.enqueue_background(
                self.background_commands.clone(),
                super::job::JobEvent::Start(request),
            );
        }
        let was_input_paused = self.attach_input_paused;
        self.attach_input_paused = !accepts_input;
        let read_enabled = !self.reads_paused && !self.attach_input_paused && !finished;
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, read_enabled);
        if was_input_paused
            && read_enabled
            && self
                .reader
                .as_ref()
                .is_some_and(ImsgReader::has_buffered_frame)
        {
            self.schedule_read_continuation(target, outbox);
        }

        let registered = self.attach_tokens.keys().copied().collect::<BTreeSet<_>>();
        for source in registered.union(&desired).copied() {
            outbox.set_protocol_interest(
                target.clone(),
                ProtocolIoSide::Attach(source),
                desired.contains(&source),
            );
        }

        if deadline != self.attach_timer_deadline {
            self.attach_timer_deadline = deadline;
            self.attach_timer_generation = self.attach_timer_generation.wrapping_add(1);
            match deadline {
                Some(deadline) => outbox.set_protocol_timer_event(
                    target.clone(),
                    deadline,
                    ProtocolEvent::AttachTimer(self.attach_timer_generation),
                ),
                None => outbox.cancel_protocol_timer(target.clone()),
            }
        }

        if finished {
            self.close_after_flush = true;
        }
        self.drive_output(target, outbox);
    }

    fn begin_fallback(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let (server_reader, mut server_writer) =
            match self.server.connect_nonblocking(FALLBACK_QUEUE_LIMIT) {
                Ok(connection) => connection,
                Err(error) => {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
            };
        while let Some(frame) = self.prelude.pop_front() {
            if server_writer.try_queue(frame).is_err() {
                self.close(
                    target,
                    ProtocolCloseReason::PreludeExceedsQueueLimit,
                    outbox,
                );
                return;
            }
        }

        let client_reader = self.reader.take().expect("protocol reader disappeared");
        let client_writer = self.writer.take().expect("protocol writer disappeared");
        self.mode = ProtocolMode::HandingOff;
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, false);
        outbox.handoff_protocol(
            target.clone(),
            client_reader,
            client_writer,
            server_reader,
            server_writer,
        );
    }

    fn begin_status_wait(&mut self, target: &ActorRef<Self>, since: u64, outbox: &mut Outbox) {
        let subscription = match self.hub.subscribe() {
            Ok(subscription) => subscription,
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                return;
            }
        };
        self.status_generation = self.status_generation.wrapping_add(1);
        let generation = self.status_generation;
        self.status_subscription = Some(subscription);
        self.operation = DirectOperation::WaitingStatus { since, generation };
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Status, true);
        outbox.set_protocol_timer(
            target.clone(),
            Instant::now() + STATUS_HEARTBEAT,
            generation,
        );
    }

    fn handle_status_ready(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let since = match &self.operation {
            DirectOperation::WaitingStatus { since, .. } => *since,
            _ => return,
        };
        let Some(subscription) = self.status_subscription.as_ref() else {
            return;
        };
        subscription.drain();
        let snapshot = self.hub.snapshot();
        if snapshot.revision > since {
            self.complete_status_wait(target, snapshot, outbox);
        }
    }

    fn handle_status_heartbeat(
        &mut self,
        target: &ActorRef<Self>,
        generation: u64,
        outbox: &mut Outbox,
    ) {
        if !matches!(
            self.operation,
            DirectOperation::WaitingStatus {
                generation: active,
                ..
            } if active == generation
        ) {
            return;
        }
        self.status_timer = None;
        let snapshot = self.hub.snapshot();
        self.complete_status_wait(target, snapshot, outbox);
    }

    fn complete_status_wait(
        &mut self,
        target: &ActorRef<Self>,
        snapshot: StatusSnapshot,
        outbox: &mut Outbox,
    ) {
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Status, false);
        outbox.cancel_protocol_timer(target.clone());
        self.start_command_work(target, CommandWork::EncodeStatus(snapshot), outbox);
    }

    fn begin_command(&mut self, target: &ActorRef<Self>, args: Vec<String>, outbox: &mut Outbox) {
        self.start_command_work(
            target,
            CommandWork::Initial {
                args,
                context: self.context.clone(),
            },
            outbox,
        );
    }

    fn start_command_work(
        &mut self,
        target: &ActorRef<Self>,
        work: CommandWork,
        outbox: &mut Outbox,
    ) {
        self.completion = None;
        self.operation = DirectOperation::Idle;
        let step = run_command_work(work, &self.state);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandStepReady(step));
    }

    fn drive_resumable_command(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let Some(mut active) = self.resumable_command.take() else {
            return;
        };
        match active.queue.drive(&self.state, COMMAND_QUEUE_BUDGET) {
            command::ResumableCommandTurn::Pending => {
                self.resumable_command = Some(active);
                outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
            }
            command::ResumableCommandTurn::Suspended(suspension) => {
                match PendingCommand::start_suspension(suspension) {
                    Ok(completion) => {
                        let complete = completion.is_complete();
                        let deadline = completion.deadline();
                        self.resumable_command = Some(active);
                        self.completion = Some(completion);
                        self.operation = DirectOperation::WaitingCommand { pending: true };
                        if complete {
                            outbox
                                .enqueue_protocol(target.clone(), ProtocolEvent::CommandCompleted);
                        } else {
                            outbox.set_protocol_interest(
                                target.clone(),
                                ProtocolIoSide::Command,
                                true,
                            );
                            if let Some(deadline) = deadline {
                                self.command_generation = self.command_generation.wrapping_add(1);
                                outbox.set_protocol_timer_event(
                                    target.clone(),
                                    deadline,
                                    ProtocolEvent::CommandTimeout(self.command_generation),
                                );
                            }
                        }
                    }
                    Err(error) => {
                        self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    }
                }
            }
            command::ResumableCommandTurn::Complete(mut result) => {
                for request in result.background_commands.drain(..) {
                    outbox.enqueue_background(
                        self.background_commands.clone(),
                        super::job::JobEvent::Start(request),
                    );
                }
                let mut transaction = active.transaction;
                if transaction.complete_group(&result) {
                    transaction.groups.clear();
                }
                self.start_command_work(target, CommandWork::Advance(transaction), outbox);
            }
        }
    }

    fn handle_command_completed(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let had_deadline = self
            .completion
            .as_ref()
            .is_some_and(|completion| completion.deadline().is_some());
        let completed = match self
            .completion
            .as_mut()
            .expect("command readiness without a completion")
            .take_result()
        {
            Ok(result) => result,
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                return;
            }
        };
        match &mut self.operation {
            DirectOperation::WaitingCommand { pending } if *pending => *pending = false,
            _ => {
                self.close(
                    target,
                    ProtocolCloseReason::Error(io::ErrorKind::InvalidData),
                    outbox,
                );
                return;
            }
        }
        self.operation = DirectOperation::Idle;
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Command, false);
        if had_deadline {
            outbox.cancel_protocol_timer(target.clone());
        }
        let Some(active) = self.resumable_command.as_mut() else {
            self.close(
                target,
                ProtocolCloseReason::Error(io::ErrorKind::InvalidData),
                outbox,
            );
            return;
        };
        active.queue.resume(completed, &self.state);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
    }

    fn handle_command_step(
        &mut self,
        target: &ActorRef<Self>,
        step: CommandStep,
        outbox: &mut Outbox,
    ) {
        match step {
            CommandStep::Complete(result) => self.begin_response(target, result, outbox),
            CommandStep::Execute {
                transaction,
                args,
                context,
            } => {
                let agents = self.hub.snapshot().panes;
                match command::start_resumable_command(&args, &self.state, &agents, &context) {
                    Ok(queue) => {
                        self.resumable_command =
                            Some(ActiveResumableCommand { transaction, queue });
                        self.drive_resumable_command(target, outbox);
                    }
                    Err(result) => {
                        let mut transaction = transaction;
                        if transaction.complete_group(&result) {
                            transaction.groups.clear();
                        }
                        self.start_command_work(target, CommandWork::Advance(transaction), outbox);
                    }
                }
            }
            CommandStep::Status { revision, body } => {
                self.operation = DirectOperation::Idle;
                self.queue_frame(
                    target,
                    Frame::new(Message::Status { revision, body }),
                    outbox,
                );
                if self.retry.is_none() {
                    outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
                    if self
                        .reader
                        .as_ref()
                        .is_some_and(ImsgReader::has_buffered_frame)
                    {
                        self.schedule_read_continuation(target, outbox);
                    }
                }
                self.drive_output(target, outbox);
            }
            CommandStep::Read {
                transaction,
                args,
                path,
            } => {
                let fd = if path.as_os_str() == "-" { 0 } else { -1 };
                let mut wire_path = path.as_os_str().as_bytes().to_vec();
                wire_path.push(0);
                self.operation = DirectOperation::WaitingClientRead {
                    transaction,
                    args,
                    data: Vec::new(),
                };
                self.queue_frame(
                    target,
                    Frame::new(Message::ReadOpen {
                        stream: FILE_STREAM,
                        fd,
                        path: wire_path,
                    }),
                    outbox,
                );
                self.drive_output(target, outbox);
            }
            CommandStep::Write {
                transaction,
                request,
            } => {
                let mut path = request.path.as_os_str().as_bytes().to_vec();
                path.push(0);
                let flags = request.flags;
                self.operation = DirectOperation::WaitingClientWriteReady {
                    transaction,
                    request,
                };
                self.queue_frame(
                    target,
                    Frame::new(Message::WriteOpen {
                        stream: FILE_STREAM,
                        fd: -1,
                        flags,
                        path,
                    }),
                    outbox,
                );
                self.drive_output(target, outbox);
            }
        }
    }

    fn handle_direct_frame(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) {
        match frame.msg {
            Message::StatusWait { since } if matches!(self.operation, DirectOperation::Idle) => {
                self.begin_status_wait(target, since, outbox);
            }
            Message::Read {
                stream: FILE_STREAM,
                data,
            } => {
                if let DirectOperation::WaitingClientRead { data: buffered, .. } =
                    &mut self.operation
                {
                    buffered.extend_from_slice(&data);
                }
            }
            Message::ReadDone {
                stream: FILE_STREAM,
                error,
            } => {
                let operation = std::mem::replace(&mut self.operation, DirectOperation::Idle);
                let DirectOperation::WaitingClientRead {
                    transaction,
                    args,
                    data,
                } = operation
                else {
                    return;
                };
                let mut context = transaction.context.clone();
                context.input_file = Some(if error == 0 { Ok(data) } else { Err(error) });
                outbox.enqueue_protocol(
                    target.clone(),
                    ProtocolEvent::CommandStepReady(CommandStep::Execute {
                        transaction,
                        args,
                        context,
                    }),
                );
            }
            Message::WriteReady {
                stream: FILE_STREAM,
                error,
            } => {
                let operation = std::mem::replace(&mut self.operation, DirectOperation::Idle);
                let DirectOperation::WaitingClientWriteReady {
                    mut transaction,
                    request,
                } = operation
                else {
                    return;
                };
                if error != 0 {
                    let mut result = CommandResult::err(format!(
                        "{}: {}\n",
                        io::Error::from_raw_os_error(error),
                        request.display_path
                    ));
                    result.continue_queue = true;
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    self.start_command_work(target, CommandWork::Advance(transaction), outbox);
                } else {
                    self.operation = DirectOperation::WritingClientFile {
                        transaction: Some(transaction),
                        request,
                        offset: 0,
                        close_generated: false,
                    };
                    self.drive_output(target, outbox);
                }
            }
            Message::WriteReady { stream, error } => {
                if let DirectOperation::Responding(response) = &mut self.operation {
                    response.acknowledge(stream, error);
                    self.drive_output(target, outbox);
                }
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown => {
                self.close(target, ProtocolCloseReason::Completed, outbox);
            }
            _ => {}
        }
    }

    fn begin_response(
        &mut self,
        target: &ActorRef<Self>,
        result: CommandResult,
        outbox: &mut Outbox,
    ) {
        self.operation = DirectOperation::Responding(CommandResponse::new(result));
        self.drive_output(target, outbox);
    }

    fn next_generated_frame(&mut self) -> GeneratedFrame {
        match &mut self.operation {
            DirectOperation::WritingClientFile {
                transaction,
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
                    )
                }
            }
            DirectOperation::Responding(response) => match response.next_frame() {
                Some(frame) => GeneratedFrame::Frame(frame),
                None if response.is_complete() => GeneratedFrame::ResponseComplete,
                None => GeneratedFrame::Blocked,
            },
            _ => GeneratedFrame::Blocked,
        }
    }

    fn drive_output(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if let Some(frame) = self.retry.take() {
            match self
                .writer
                .as_mut()
                .expect("protocol writer disappeared")
                .try_queue(frame)
            {
                Ok(()) => {
                    self.reads_paused = false;
                    if !self.attach_input_paused {
                        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
                    }
                    if !self.attach_input_paused
                        && self
                            .reader
                            .as_ref()
                            .is_some_and(ImsgReader::has_buffered_frame)
                    {
                        self.schedule_read_continuation(target, outbox);
                    }
                }
                Err(error)
                    if self
                        .writer
                        .as_ref()
                        .expect("protocol writer disappeared")
                        .has_pending() =>
                {
                    self.retry = Some(error.into_frame());
                    outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, true);
                    return;
                }
                Err(_) => {
                    self.close(target, ProtocolCloseReason::FrameExceedsQueueLimit, outbox);
                    return;
                }
            }
        }

        loop {
            match self.next_generated_frame() {
                GeneratedFrame::Frame(frame) => {
                    if !self.queue_frame(target, frame, outbox) {
                        return;
                    }
                }
                GeneratedFrame::ClientFileComplete(mut transaction) => {
                    let result = CommandResult::ok("");
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    self.operation = DirectOperation::Idle;
                    self.start_command_work(target, CommandWork::Advance(transaction), outbox);
                    if self.retry.is_some() {
                        return;
                    }
                }
                GeneratedFrame::ResponseComplete => {
                    self.close_after_flush = true;
                    break;
                }
                GeneratedFrame::Blocked => break,
            }
        }

        let pending = self
            .writer
            .as_ref()
            .is_some_and(NonblockingImsgWriter::has_pending);
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, pending);
        if self.close_after_flush && !pending && self.retry.is_none() {
            self.close(target, ProtocolCloseReason::Completed, outbox);
        }
    }

    fn queue_frame(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) -> bool {
        log_frame(Direction::ServerToClient, &frame);
        match self
            .writer
            .as_mut()
            .expect("protocol writer disappeared")
            .try_queue(frame)
        {
            Ok(()) => {
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, true);
                true
            }
            Err(error)
                if self
                    .writer
                    .as_ref()
                    .expect("protocol writer disappeared")
                    .has_pending() =>
            {
                self.retry = Some(error.into_frame());
                self.reads_paused = true;
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Write, true);
                false
            }
            Err(_) => {
                self.close(target, ProtocolCloseReason::FrameExceedsQueueLimit, outbox);
                false
            }
        }
    }

    fn handle_writable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        match self
            .writer
            .as_mut()
            .expect("protocol writer disappeared")
            .try_flush()
        {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                return;
            }
        }
        self.drive_output(target, outbox);
        if matches!(self.mode, ProtocolMode::Attach(_)) && self.status.close_reason().is_none() {
            self.sync_attach(target, outbox);
        }
    }

    fn schedule_read_continuation(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.mark_work_queued(ProtocolIoSide::Read) {
            outbox.enqueue_protocol(target.clone(), ProtocolEvent::ReadContinuation);
        }
    }

    fn close(&mut self, target: &ActorRef<Self>, reason: ProtocolCloseReason, outbox: &mut Outbox) {
        if matches!(
            self.mode,
            ProtocolMode::HandingOff | ProtocolMode::Fallback(_)
        ) {
            return;
        }
        self.status.close_reason.set(Some(reason));
        for side in [
            ProtocolIoSide::Read,
            ProtocolIoSide::Write,
            ProtocolIoSide::Command,
            ProtocolIoSide::Status,
        ] {
            outbox.set_protocol_interest(target.clone(), side, false);
        }
        for source in self.control_tokens.keys().copied().collect::<Vec<_>>() {
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Control(source), false);
        }
        for source in self.attach_tokens.keys().copied().collect::<Vec<_>>() {
            outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Attach(source), false);
        }
        if let ProtocolMode::Attach(attach) = &mut self.mode {
            attach.shutdown();
        }
        self.control_timer_deadline = None;
        self.attach_timer_deadline = None;
        outbox.cancel_protocol_timer(target.clone());
        outbox.stop_protocol(target.clone());
    }
}

enum GeneratedFrame {
    Frame(Frame),
    ClientFileComplete(CommandTransaction),
    ResponseComplete,
    Blocked,
}

/// One validated command line, possibly split around client-side file work.
pub(crate) struct CommandTransaction {
    groups: VecDeque<Vec<String>>,
    output: CommandResult,
    context: ClientContext,
}

struct ActiveResumableCommand {
    transaction: CommandTransaction,
    queue: command::ResumableCommandQueue,
}

impl CommandTransaction {
    fn new(groups: Vec<Vec<String>>, context: ClientContext) -> Self {
        Self {
            groups: groups.into(),
            output: CommandResult::ok(""),
            context,
        }
    }

    /// Merge a group result and report whether the remaining group tail stops.
    fn complete_group(&mut self, result: &CommandResult) -> bool {
        let exit = result.exit;
        self.output.continue_queue |= result.continue_queue;
        self.output.append_stdout(result);
        self.output.stderr.push_str(&result.stderr);
        if self.output.exit == 0 || exit != 0 {
            self.output.exit = exit;
        }
        exit != 0 && !result.continue_queue
    }
}

enum CommandWork {
    Initial {
        args: Vec<String>,
        context: ClientContext,
    },
    Advance(CommandTransaction),
    EncodeStatus(StatusSnapshot),
}

pub(crate) enum CommandStep {
    Complete(CommandResult),
    Status {
        revision: u64,
        body: Vec<u8>,
    },
    Read {
        transaction: CommandTransaction,
        args: Vec<String>,
        path: PathBuf,
    },
    Write {
        transaction: CommandTransaction,
        request: command::ClientFileWrite,
    },
    Execute {
        transaction: CommandTransaction,
        args: Vec<String>,
        context: ClientContext,
    },
}

fn run_command_work(work: CommandWork, state: &Arc<Mutex<ServerState>>) -> CommandStep {
    if let CommandWork::EncodeStatus(snapshot) = work {
        let body = match state.lock() {
            Ok(state) => command::encode_status_body(&state, &snapshot.panes),
            Err(_) => {
                return CommandStep::Complete(CommandResult::err("server state poisoned\n"));
            }
        };
        return CommandStep::Status {
            revision: snapshot.revision,
            body,
        };
    }
    match work {
        CommandWork::Initial { args, context } => {
            let aliases = match state.lock() {
                Ok(state) => state.command_aliases(),
                Err(_) => {
                    return CommandStep::Complete(CommandResult::err("server state poisoned\n"));
                }
            };
            let groups = match command::command_line_groups(&args, &aliases) {
                Ok(groups) => groups,
                Err(result) => return CommandStep::Complete(result),
            };
            let groups = if groups.len() > 1
                && groups
                    .iter()
                    .any(|group| command::uses_client_file_protocol(group))
            {
                groups
            } else {
                vec![args]
            };
            advance_command_transaction(CommandTransaction::new(groups, context), state)
        }
        CommandWork::Advance(transaction) => advance_command_transaction(transaction, state),
        CommandWork::EncodeStatus(_) => unreachable!("status work returned above"),
    }
}

fn advance_command_transaction(
    mut transaction: CommandTransaction,
    state: &Arc<Mutex<ServerState>>,
) -> CommandStep {
    loop {
        let Some(args) = transaction.groups.pop_front() else {
            return CommandStep::Complete(transaction.output);
        };

        let file_write = match state.lock() {
            Ok(state) => command::save_buffer_client_request(&args, &state, &transaction.context),
            Err(_) => Some(Err(CommandResult::err("server state poisoned\n"))),
        };
        if let Some(request) = file_write {
            match request {
                Err(result) => {
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    continue;
                }
                Ok(request) => {
                    return CommandStep::Write {
                        transaction,
                        request,
                    };
                }
            }
        }

        if let Some(path) = command::client_input_path(&args, &transaction.context) {
            return CommandStep::Read {
                transaction,
                args,
                path,
            };
        }

        let context = transaction.context.clone();
        return CommandStep::Execute {
            transaction,
            args,
            context,
        };
    }
}

enum PendingCommand {
    Worker {
        completion: UnixStream,
        result: Arc<Mutex<Option<command::CommandSuspensionResult>>>,
    },
    PaneOutput(command::PaneOutputSuspension),
}

impl PendingCommand {
    fn start_suspension(suspension: command::CommandSuspension) -> io::Result<Self> {
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

    fn take_result(&mut self) -> io::Result<command::CommandSuspensionResult> {
        let Self::Worker { completion, result } = self else {
            let Self::PaneOutput(wait) = self else {
                unreachable!();
            };
            return Ok(wait.complete());
        };
        let mut byte = [0u8; 1];
        completion.read_exact(&mut byte)?;
        result
            .lock()
            .map_err(|_| io::Error::other("command result poisoned"))?
            .take()
            .ok_or_else(|| io::Error::other("command completed without a result"))
    }
}

struct CommandResponse {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: i32,
    phase: ResponsePhase,
}

enum ResponsePhase {
    OpenStdout,
    WaitStdout,
    WriteStdout(usize),
    CloseStdout,
    OpenStderr,
    WaitStderr,
    WriteStderr(usize),
    CloseStderr,
    Exit,
    Complete,
}

impl CommandResponse {
    fn new(result: CommandResult) -> Self {
        Self {
            stdout: result.stdout_data().to_vec(),
            stderr: result.stderr.into_bytes(),
            exit: result.exit,
            phase: ResponsePhase::OpenStdout,
        }
    }

    fn acknowledge(&mut self, stream: i32, error: i32) {
        self.phase = match (&self.phase, stream) {
            (ResponsePhase::WaitStdout, 1) if error == 0 => ResponsePhase::WriteStdout(0),
            (ResponsePhase::WaitStdout, 1) => ResponsePhase::CloseStdout,
            (ResponsePhase::WaitStderr, 2) if error == 0 => ResponsePhase::WriteStderr(0),
            (ResponsePhase::WaitStderr, 2) => ResponsePhase::CloseStderr,
            _ => return,
        };
    }

    fn next_frame(&mut self) -> Option<Frame> {
        loop {
            match &mut self.phase {
                ResponsePhase::OpenStdout if self.stdout.is_empty() => {
                    self.phase = ResponsePhase::OpenStderr;
                }
                ResponsePhase::OpenStdout => {
                    self.phase = ResponsePhase::WaitStdout;
                    return Some(Frame::new(Message::WriteOpen {
                        stream: 1,
                        fd: 1,
                        flags: 0,
                        path: Vec::new(),
                    }));
                }
                ResponsePhase::WaitStdout | ResponsePhase::WaitStderr => return None,
                ResponsePhase::WriteStdout(offset) if *offset < self.stdout.len() => {
                    let end = (*offset + OUTPUT_CHUNK).min(self.stdout.len());
                    let frame = Frame::new(Message::Write {
                        stream: 1,
                        data: self.stdout[*offset..end].to_vec(),
                    });
                    *offset = end;
                    return Some(frame);
                }
                ResponsePhase::WriteStdout(_) => {
                    self.phase = ResponsePhase::CloseStdout;
                }
                ResponsePhase::CloseStdout => {
                    self.phase = ResponsePhase::OpenStderr;
                    return Some(Frame::new(Message::WriteClose { stream: 1 }));
                }
                ResponsePhase::OpenStderr if self.stderr.is_empty() => {
                    self.phase = ResponsePhase::Exit;
                }
                ResponsePhase::OpenStderr => {
                    self.phase = ResponsePhase::WaitStderr;
                    return Some(Frame::new(Message::WriteOpen {
                        stream: 2,
                        fd: 2,
                        flags: 0,
                        path: Vec::new(),
                    }));
                }
                ResponsePhase::WriteStderr(offset) if *offset < self.stderr.len() => {
                    let end = (*offset + OUTPUT_CHUNK).min(self.stderr.len());
                    let frame = Frame::new(Message::Write {
                        stream: 2,
                        data: self.stderr[*offset..end].to_vec(),
                    });
                    *offset = end;
                    return Some(frame);
                }
                ResponsePhase::WriteStderr(_) => {
                    self.phase = ResponsePhase::CloseStderr;
                }
                ResponsePhase::CloseStderr => {
                    self.phase = ResponsePhase::Exit;
                    return Some(Frame::new(Message::WriteClose { stream: 2 }));
                }
                ResponsePhase::Exit => {
                    self.phase = ResponsePhase::Complete;
                    return Some(Frame::new(Message::Exit(Some(self.exit))));
                }
                ResponsePhase::Complete => return None,
            }
        }
    }

    fn is_complete(&self) -> bool {
        matches!(self.phase, ResponsePhase::Complete)
    }
}

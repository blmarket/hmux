//! Event-loop-owned tmux command-client protocol state.

use std::cell::Cell;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::integration::status::StatusHub;
use crate::tmux::codec::{encode_bytes, ImsgReader, NonblockingImsgWriter, MAX_IMSGSIZE};
use crate::tmux::introspect::{log_frame, Direction};
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::native::command::{self, ClientContext, CommandResult};
use crate::tmux::native::state::ServerState;
use crate::tmux::native::NativeServer;
use crate::tmux::traits::NonblockingFrameWriter;

use super::actor::ActorRef;
use super::client::READ_FRAME_BUDGET;
use super::driver::{Outbox, PairingHandle};
use super::pairing::PairingCloseReason;
use super::reactor::Token;

const CLIENT_CONTROL: i64 = 0x2000;
const FALLBACK_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;
const FILE_STREAM: i32 = 3;
const OUTPUT_CHUNK: usize = 8 * 1024;

/// A readiness source owned by a protocol actor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolIoSide {
    Read,
    Write,
    Command,
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
    CommandStepReady(CommandStep),
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
    Responding(CommandResponse),
}

/// Direct command-client state plus an optional compatibility handoff.
pub(crate) struct ProtocolClient {
    reader: Option<ImsgReader>,
    writer: Option<NonblockingImsgWriter>,
    server: NativeServer,
    state: Arc<Mutex<ServerState>>,
    hub: StatusHub,
    mode: ProtocolMode,
    context: ClientContext,
    control_mode: bool,
    prelude: VecDeque<Frame>,
    prelude_bytes: usize,
    operation: DirectOperation,
    retry: Option<Frame>,
    completion: Option<PendingCommand>,
    read_token: Option<Token>,
    write_token: Option<Token>,
    command_token: Option<Token>,
    read_work_queued: bool,
    write_work_queued: bool,
    command_work_queued: bool,
    reads_paused: bool,
    close_after_flush: bool,
    status: ProtocolStatus,
}

impl ProtocolClient {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: NativeServer,
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
                mode: ProtocolMode::Identifying,
                context: ClientContext {
                    wait_for_interactions: true,
                    ..ClientContext::default()
                },
                control_mode: false,
                prelude: VecDeque::new(),
                prelude_bytes: 0,
                operation: DirectOperation::Idle,
                retry: None,
                completion: None,
                read_token: None,
                write_token: None,
                command_token: None,
                read_work_queued: false,
                write_work_queued: false,
                command_work_queued: false,
                reads_paused: false,
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
        }
    }

    pub(crate) fn token(&self, side: ProtocolIoSide) -> Option<Token> {
        match side {
            ProtocolIoSide::Read => self.read_token,
            ProtocolIoSide::Write => self.write_token,
            ProtocolIoSide::Command => self.command_token,
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
        }
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
        };
        if *queued || (side == ProtocolIoSide::Read && self.reads_paused) {
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
            ProtocolMode::Identifying | ProtocolMode::Direct | ProtocolMode::HandingOff => true,
        }
    }

    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        matches!(self.mode, ProtocolMode::Direct)
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
            ProtocolEvent::CommandStepReady(step) => {
                self.handle_command_step(target, step, outbox);
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
                ProtocolMode::HandingOff | ProtocolMode::Fallback(_) => return,
            }
            if self.reader.is_none() || self.retry.is_some() || self.close_after_flush {
                return;
            }
        }

        self.schedule_read_continuation(target, outbox);
    }

    fn handle_identifying(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) {
        self.observe_identify(&frame);
        match &frame.msg {
            Message::Command(args)
                if !self.control_mode && command::classify(args) == command::Intent::Command =>
            {
                let args = args.clone();
                self.prelude.clear();
                self.prelude_bytes = 0;
                self.mode = ProtocolMode::Direct;
                self.begin_command(target, args, outbox);
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
        match PendingCommand::start(work, Arc::clone(&self.state), self.hub.clone()) {
            Ok(completion) => {
                self.completion = Some(completion);
                self.operation = DirectOperation::WaitingCommand { pending: true };
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Command, true);
            }
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            }
        }
    }

    fn handle_command_completed(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let step = match self
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
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandStepReady(step));
    }

    fn handle_command_step(
        &mut self,
        target: &ActorRef<Self>,
        step: CommandStep,
        outbox: &mut Outbox,
    ) {
        match step {
            CommandStep::Complete(result) => self.begin_response(target, result, outbox),
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
                self.start_command_work(
                    target,
                    CommandWork::Execute {
                        transaction,
                        args,
                        context,
                    },
                    outbox,
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
                    outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, true);
                    if self
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
        ] {
            outbox.set_protocol_interest(target.clone(), side, false);
        }
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
    Execute {
        transaction: CommandTransaction,
        args: Vec<String>,
        context: ClientContext,
    },
}

pub(crate) enum CommandStep {
    Complete(CommandResult),
    Read {
        transaction: CommandTransaction,
        args: Vec<String>,
        path: PathBuf,
    },
    Write {
        transaction: CommandTransaction,
        request: command::ClientFileWrite,
    },
}

fn run_command_work(
    work: CommandWork,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
) -> CommandStep {
    let agents = hub.snapshot().panes;
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
            advance_command_transaction(CommandTransaction::new(groups, context), state, &agents)
        }
        CommandWork::Advance(transaction) => {
            advance_command_transaction(transaction, state, &agents)
        }
        CommandWork::Execute {
            mut transaction,
            args,
            context,
        } => {
            let result = command::run_with_context(&args, state, &agents, &context);
            if transaction.complete_group(&result) {
                transaction.groups.clear();
            }
            advance_command_transaction(transaction, state, &agents)
        }
    }
}

fn advance_command_transaction(
    mut transaction: CommandTransaction,
    state: &Arc<Mutex<ServerState>>,
    agents: &crate::integration::status::PaneAgents,
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

        let result = command::run_with_context(&args, state, agents, &transaction.context);
        if transaction.complete_group(&result) {
            transaction.groups.clear();
        }
    }
}

struct PendingCommand {
    completion: UnixStream,
    result: Arc<Mutex<Option<CommandStep>>>,
}

impl PendingCommand {
    fn start(
        work: CommandWork,
        state: Arc<Mutex<ServerState>>,
        hub: StatusHub,
    ) -> io::Result<Self> {
        let (completion, mut signal) = UnixStream::pair()?;
        completion.set_nonblocking(true)?;
        let result = Arc::new(Mutex::new(None));
        let worker_result = Arc::clone(&result);
        thread::spawn(move || {
            let completed = run_command_work(work, &state, &hub);
            if let Ok(mut result) = worker_result.lock() {
                *result = Some(completed);
            }
            let _ = signal.write_all(&[1]);
        });
        Ok(Self { completion, result })
    }

    fn fd(&self) -> BorrowedFd<'_> {
        self.completion.as_fd()
    }

    fn take_result(&mut self) -> io::Result<CommandStep> {
        let mut byte = [0u8; 1];
        self.completion.read_exact(&mut byte)?;
        self.result
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

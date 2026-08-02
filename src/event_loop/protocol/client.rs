use super::*;
use crate::server::command;

pub(crate) const CLIENT_CONTROL: i64 = 0x2000;

const IDENTIFY_LIMIT: usize = MAX_IMSGSIZE * 64;

/// Maximum decoded frames handled by one read event.
pub(crate) const READ_FRAME_BUDGET: usize = 32;

pub(super) const FILE_STREAM: i32 = 3;

pub(crate) const COMMAND_QUEUE_BUDGET: usize = 64;

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
    Shutdown,
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

/// Event-loop-owned command, control-mode, and interactive-attach protocol state.
pub(crate) struct ProtocolClient {
    pub(super) reader: ImsgReader,
    writer: NonblockingImsgWriter,
    pub(super) state: Arc<Mutex<ServerState>>,
    pub(super) hub: StatusHub,
    pub(super) background_commands: ActorRef<BackgroundCommands>,
    pub(super) mode: ProtocolMode,
    pub(super) context: ClientContext,
    pub(super) client_tty: ClientTty,
    pub(super) identify_bytes: usize,
    control_mode: bool,
    pub(super) operation: DirectOperation,
    pub(super) completion: Option<PendingCommand>,
    pub(super) resumable_command: Option<ActiveResumableCommand>,
    read_token: Option<Token>,
    write_token: Option<Token>,
    command_token: Option<Token>,
    pub(super) control_tokens: BTreeMap<EventControlSource, Token>,
    pub(super) attach_tokens: BTreeMap<EventAttachSource, Token>,
    status_timer: Option<TimerId>,
    pub(super) command_generation: u64,
    pub(super) control_timer_deadline: Option<Instant>,
    pub(super) control_timer_generation: u64,
    pub(super) attach_timer_deadline: Option<Instant>,
    pub(super) attach_timer_generation: u64,
    work_queued: BTreeSet<ProtocolIoSide>,
    pub(super) reads_paused: bool,
    pub(super) attach_input_paused: bool,
    pub(super) close_after_flush: bool,
    status: ProtocolStatus,
}

enum GeneratedFrame {
    Frame(Frame),
    ClientFileComplete(CommandTransaction),
    ResponseComplete,
    Blocked,
}

pub(super) enum ProtocolMode {
    Identifying,
    Direct,
    Control(Box<EventControlClient>),
    Attach(Box<EventAttachClient>),
}

/// Why the connection actor stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CloseReason {
    PeerClosed,
    Error(io::ErrorKind),
    Shutdown,
    FrameExceedsQueueLimit,
}

/// Events delivered to the connection actor.
pub(crate) enum ClientIoEvent {
    Start,
    Readable,
    ReadContinuation,
    Writable,
    Send(Frame),
    Shutdown,
}

/// Events delivered by the I/O actor to its current domain placeholder.
pub(crate) enum ClientInboxEvent {
    Frame(Frame),
    Closed(CloseReason),
}

/// Minimal domain-side actor used while the real client actor is migrated.
#[derive(Default)]
pub(crate) struct ClientInbox {
    frames: VecDeque<Frame>,
    close_reason: Option<CloseReason>,
}

impl ClientInbox {
    pub(crate) fn len(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop_front()
    }

    pub(crate) fn close_reason(&self) -> Option<CloseReason> {
        self.close_reason
    }

    fn handle(&mut self, event: ClientInboxEvent) {
        match event {
            ClientInboxEvent::Frame(frame) => self.frames.push_back(frame),
            ClientInboxEvent::Closed(reason) => self.close_reason = Some(reason),
        }
    }
}

/// Read/write state for one client connection.
pub(crate) struct ClientIo {
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    inbox: ActorRef<ClientInbox>,
    read_token: Option<Token>,
    write_token: Option<Token>,
    read_work_queued: bool,
    write_work_queued: bool,
    send_work_queued: bool,
    reads_paused: bool,
    closed: bool,
}

impl ClientIo {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        inbox: ActorRef<ClientInbox>,
    ) -> Self {
        Self {
            reader,
            writer,
            inbox,
            read_token: None,
            write_token: None,
            read_work_queued: false,
            write_work_queued: false,
            send_work_queued: false,
            reads_paused: false,
            closed: false,
        }
    }

    pub(crate) fn read_fd(&self) -> BorrowedFd<'_> {
        self.reader.as_fd()
    }

    pub(crate) fn write_fd(&self) -> BorrowedFd<'_> {
        self.writer.as_fd()
    }

    pub(crate) fn read_token(&self) -> Option<Token> {
        self.read_token
    }

    pub(crate) fn write_token(&self) -> Option<Token> {
        self.write_token
    }

    pub(crate) fn set_read_token(&mut self, token: Option<Token>) {
        self.read_token = token;
    }

    pub(crate) fn set_write_token(&mut self, token: Option<Token>) {
        self.write_token = token;
    }

    pub(crate) fn mark_read_work_queued(&mut self) -> bool {
        if self.closed || self.reads_paused || self.read_work_queued {
            return false;
        }
        self.read_work_queued = true;
        true
    }

    pub(crate) fn mark_write_work_queued(&mut self) -> bool {
        if self.closed || self.write_work_queued {
            return false;
        }
        self.write_work_queued = true;
        true
    }

    pub(crate) fn mark_send_work_queued(&mut self) -> bool {
        if self.closed || !self.writer.is_below_high_water() || self.send_work_queued {
            return false;
        }
        self.send_work_queued = true;
        true
    }

    pub(crate) fn reads_paused(&self) -> bool {
        self.reads_paused
    }

    pub(crate) fn handle(
        &mut self,
        target: &ActorRef<Self>,
        event: ClientIoEvent,
        outbox: &mut Outbox,
    ) {
        if self.closed {
            return;
        }

        match event {
            ClientIoEvent::Start => outbox.set_read_interest(target.clone(), true),
            ClientIoEvent::Readable => {
                self.read_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ClientIoEvent::ReadContinuation => {
                self.read_work_queued = false;
                self.handle_readable(target, outbox);
            }
            ClientIoEvent::Writable => {
                self.write_work_queued = false;
                self.handle_writable(target, outbox);
            }
            ClientIoEvent::Send(frame) => {
                self.send_work_queued = false;
                self.handle_send(target, frame, outbox);
            }
            ClientIoEvent::Shutdown => self.close(target, CloseReason::Shutdown, outbox),
        }
    }

    fn handle_readable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused {
            return;
        }

        for _ in 0..READ_FRAME_BUDGET {
            match NonblockingFrameReader::try_recv(&mut self.reader) {
                Ok(frame) => outbox.enqueue(Envelope::ClientInbox {
                    target: self.inbox.clone(),
                    event: ClientInboxEvent::Frame(frame),
                }),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    self.close(target, CloseReason::PeerClosed, outbox);
                    return;
                }
                Err(error) => {
                    self.close(target, CloseReason::Error(error.kind()), outbox);
                    return;
                }
            }
        }

        self.schedule_read_continuation(target, outbox);
    }

    fn handle_send(&mut self, target: &ActorRef<Self>, frame: Frame, outbox: &mut Outbox) {
        match self.writer.try_queue(frame) {
            Ok(()) => {
                outbox.set_write_interest(target.clone(), true);
                if !self.writer.is_below_high_water() {
                    self.reads_paused = true;
                    outbox.set_read_interest(target.clone(), false);
                }
            }
            Err(_) => self.close(target, CloseReason::FrameExceedsQueueLimit, outbox),
        }
    }

    fn handle_writable(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        match self.writer.try_flush() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                self.close(target, CloseReason::Error(error.kind()), outbox);
                return;
            }
        }

        if self.reads_paused && self.writer.is_below_high_water() {
            self.reads_paused = false;
            outbox.set_read_interest(target.clone(), true);
            if self.reader.has_buffered_frame() {
                self.schedule_read_continuation(target, outbox);
            }
        }

        outbox.set_write_interest(target.clone(), self.writer.has_pending());
    }

    fn schedule_read_continuation(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.mark_read_work_queued() {
            outbox.enqueue(Envelope::ClientIo {
                target: target.clone(),
                event: ClientIoEvent::ReadContinuation,
            });
        }
    }

    fn close(&mut self, target: &ActorRef<Self>, reason: CloseReason, outbox: &mut Outbox) {
        self.closed = true;
        outbox.set_read_interest(target.clone(), false);
        outbox.set_write_interest(target.clone(), false);
        outbox.enqueue(Envelope::ClientInbox {
            target: self.inbox.clone(),
            event: ClientInboxEvent::Closed(reason),
        });
        outbox.stop_client(target.clone());
    }
}

pub(crate) fn dispatch_inbox(target: &ActorRef<ClientInbox>, event: ClientInboxEvent) {
    target.with_mut(|inbox| inbox.handle(event));
}

impl ProtocolClient {
    pub(crate) fn new(
        reader: ImsgReader,
        writer: NonblockingImsgWriter,
        server: Server,
        background_commands: ActorRef<BackgroundCommands>,
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
                mode: ProtocolMode::Identifying,
                context: ClientContext {
                    wait_for_interactions: true,
                    defer_background_commands: true,
                    defer_attach_commands: true,
                    ..ClientContext::default()
                },
                client_tty: ClientTty::new(),
                identify_bytes: 0,
                control_mode: false,
                operation: DirectOperation::Idle,
                completion: None,
                resumable_command: None,
                read_token: None,
                write_token: None,
                command_token: None,
                control_tokens: BTreeMap::new(),
                attach_tokens: BTreeMap::new(),
                status_timer: None,
                command_generation: 0,
                control_timer_deadline: None,
                control_timer_generation: 0,
                attach_timer_deadline: None,
                attach_timer_generation: 0,
                work_queued: BTreeSet::new(),
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
            ProtocolIoSide::Read => Some(self.reader.as_fd()),
            ProtocolIoSide::Write => Some(self.writer.as_fd()),
            ProtocolIoSide::Command => self.completion.as_ref().map(PendingCommand::fd),
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

    pub(crate) fn mark_work_queued(&mut self, side: ProtocolIoSide) -> bool {
        if side == ProtocolIoSide::Read && (self.reads_paused || self.attach_input_paused) {
            return false;
        }
        self.work_queued.insert(side)
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

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.status.close_reason()
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
            ProtocolEvent::ControlReady(source) => {
                self.work_queued.remove(&ProtocolIoSide::Control(source));
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
                self.work_queued.remove(&ProtocolIoSide::Attach(source));
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
        if self.reads_paused {
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
                self.client_tty = ClientTty::new();
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
            }
            if self.reads_paused || self.attach_input_paused || self.close_after_flush {
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
            if frame_bytes > IDENTIFY_LIMIT.saturating_sub(self.identify_bytes) {
                self.close(target, ProtocolCloseReason::IdentifyExceedsLimit, outbox);
                return;
            }
            self.identify_bytes += frame_bytes;
        }
        self.observe_identify(&mut frame);
        match &frame.msg {
            Message::Command(args) if self.control_mode => {
                let args = args.clone();
                self.begin_control(target, args, outbox);
            }
            Message::Command(args)
                if !self.control_mode && command::classify(args) == command::Intent::Command =>
            {
                let args = args.clone();
                self.client_tty = ClientTty::new();
                self.identify_bytes = 0;
                self.mode = ProtocolMode::Direct;
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

    fn observe_identify(&mut self, frame: &mut Frame) {
        match &frame.msg {
            Message::IdentifyFlags(flags) => {
                self.control_mode |= i64::from(*flags) & CLIENT_CONTROL != 0;
                self.client_tty.flags |= i64::from(*flags);
            }
            Message::IdentifyLongFlags(flags) | Message::Flags(flags) => {
                self.control_mode |= *flags & CLIENT_CONTROL != 0;
                self.client_tty.flags |= *flags;
            }
            Message::IdentifyTerminfo(capability) => {
                self.client_tty.terminfo.push(capability.clone());
            }
            Message::IdentifyFeatures(features) => {
                self.client_tty.features |= *features as u32;
            }
            Message::IdentifyTerm(term) => {
                self.client_tty.term = Some(term.clone());
            }
            Message::IdentifyTtyName(tty_name) => {
                self.context.tty_name = Some(tty_name.clone());
                self.client_tty.tty_name = Some(tty_name.clone());
            }
            Message::IdentifyClientPid(pid) => {
                self.context.client_pid = Some(*pid);
                self.client_tty.client_pid = Some(*pid);
            }
            Message::IdentifyCwd(cwd) => {
                self.context.cwd = Some(cwd.into());
            }
            Message::IdentifyEnviron(entry) => {
                self.context.environment.push(entry.clone());
            }
            Message::IdentifyStdin => {
                self.client_tty.stdin = frame.fd.take();
            }
            Message::IdentifyStdout => {
                self.client_tty.stdout = frame.fd.take();
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

    pub(super) fn drive_output(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        if self.reads_paused && self.writer_is_below_high_water() {
            self.reads_paused = false;
            if !self.attach_input_paused {
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
                GeneratedFrame::ClientFileComplete(mut transaction) => {
                    let result = CommandResult::ok("");
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    self.operation = DirectOperation::Idle;
                    self.start_command_work(target, CommandWork::Advance(transaction), outbox);
                }
                GeneratedFrame::ResponseComplete => {
                    self.close_after_flush = true;
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
        if self.close_after_flush && !pending {
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
            match self.mode {
                ProtocolMode::Control(_) => self.sync_control(target, outbox),
                ProtocolMode::Attach(_) => self.sync_attach(target, outbox),
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

    pub(crate) fn status_timer(&self) -> Option<TimerId> {
        self.status_timer
    }

    pub(crate) fn set_status_timer(&mut self, timer: Option<TimerId>) {
        self.status_timer = timer;
    }
}

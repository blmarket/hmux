//! Interactive attach, served as a task.
//!
//! The task owns the session, the descriptors it reads and writes, and the
//! sequence it runs through. A source the session does not want this turn is an
//! arm left out of the wait rather than a registration handed back; a key
//! binding's command is a future awaited where it was started, with the rest of
//! the session running behind it only while the client is the one being asked;
//! and the end of the attach is the epilogue after the loop.

use std::collections::VecDeque;
use std::io;
use std::os::fd::{BorrowedFd, RawFd};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::integration::status::StatusHub;
use crate::server::attach::FrameSink;
use crate::server::attach::{
    self, AttachCommandContinuation, AttachDrive, AttachFinishReason, AttachPrepared,
    AttachSession, AttachStartFailure, AttachWaitReady, AttachWaitSources, ClientTty,
};
use crate::server::command::{self, ClientContext};
use crate::server::state::SharedState;
use crate::sync::{maybe, race, select, yield_now, Either, Notify, WakeFn};
use crate::tmux::codec::{encode_bytes, MAX_IMSGSIZE};
use crate::tmux::message::{Frame, Message};
use crate::tmux::traits::NonblockingFrameReader;
use hmux_rt::{sleep_until, AsyncFd, Interest, TaskHandle};

use super::wire::Wire;
use super::{ClientRuntime, ProtocolCloseReason};

const ATTACH_QUEUE_LIMIT: usize = MAX_IMSGSIZE * 64;
/// How many turns the session may take in hand before the loop gets one.
const IMMEDIATE_TURN_BUDGET: usize = 64;
const COMMAND_QUEUE_BUDGET: usize = 64;
/// Frames taken from the socket before the task gives the loop a turn.
const READ_FRAME_BUDGET: usize = 32;
/// How long the server waits for the client to answer what it was just told.
const FINISH_ACK_TIMEOUT: Duration = Duration::from_secs(2);

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

/// Interactive attach state driven directly by one protocol task.
pub(super) struct EventAttachClient {
    session: AttachSession,
    state: SharedState,
    hub: StatusHub,
    input: AttachInput,
    output: AttachOutput,
    background_commands: Vec<command::BackgroundCommandRequest>,
    tasks: TaskHandle,
}

/// One key binding's command running as a task, with what the session needs to
/// take the result back.
struct StartedAttachCommand {
    task: command::QueuedCommand,
    continuation: AttachCommandContinuation,
}

impl EventAttachClient {
    fn new(
        args: &[String],
        client_tty: ClientTty,
        state: SharedState,
        hub: StatusHub,
        context: &ClientContext,
        tasks: TaskHandle,
    ) -> Result<Self, AttachStartError> {
        let mut output = AttachOutput::new();
        let session =
            attach::start_attach_session(args, client_tty, &state, &hub, context, &mut output)
                .map_err(AttachStartError::from_failure)?;
        Ok(Self {
            session,
            state,
            hub,
            input: AttachInput::new(),
            output,
            background_commands: Vec::new(),
            tasks,
        })
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.output.pop()
    }

    /// Whether the engine can hold another frame from the client.
    fn accepts_frames(&self) -> bool {
        self.input.is_below_high_water()
    }

    fn push_frame(&mut self, frame: Frame) {
        self.input.push(frame);
    }

    fn take_background_commands(&mut self) -> Vec<command::BackgroundCommandRequest> {
        std::mem::take(&mut self.background_commands)
    }

    /// What the session wants to wait on, after doing the housekeeping a turn
    /// starts with.
    fn prepare(&mut self) -> io::Result<AttachPrepared> {
        let buffered = !self.input.is_empty();
        self.session.prepare_wait(&self.state, buffered)
    }

    /// Give the session a turn with whatever became ready.
    fn drive(&mut self, ready: AttachWaitReady) -> io::Result<Option<AttachFinishReason>> {
        match self.session.drive_ready(
            &self.state,
            &self.hub,
            ready,
            &mut self.input,
            &mut self.output,
        )? {
            AttachDrive::Continue => Ok(None),
            AttachDrive::Finish(reason) => Ok(Some(reason)),
        }
    }

    /// Start the next command a key binding deferred, if there is one.
    ///
    /// A command that cannot start reports its failure where it was asked for,
    /// so `None` means only that nothing is left to run.
    fn start_next_command(&mut self) -> Option<StartedAttachCommand> {
        loop {
            let request = self.session.take_command_request()?;
            let agents = self.hub.snapshot().panes;
            let queue = match request.source {
                command::DeferredCommand::Command(command) => {
                    tracing::debug!("starting attached-client command");
                    command::start_resumable_command(
                        command,
                        &self.state,
                        &agents,
                        &request.context,
                    )
                }
                command::DeferredCommand::Argv(ref args) => {
                    tracing::debug!(?args, "starting attached-client command");
                    command::start_resumable_command_argv(
                        args,
                        &self.state,
                        &agents,
                        &request.context,
                    )
                }
                command::DeferredCommand::Line { ref line, ref tail } => {
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
                command::spawn_queue(
                    &self.tasks,
                    queue,
                    Rc::clone(&self.state),
                    COMMAND_QUEUE_BUDGET,
                )
                .map_err(|error| command::CommandResult::err(format!("{error}\n")))
            });
            match queue {
                Ok(task) => {
                    return Some(StartedAttachCommand {
                        task,
                        continuation: request.continuation,
                    })
                }
                Err(result) => {
                    self.session
                        .complete_command(request.continuation, result, &self.state)
                }
            }
        }
    }

    /// Take a finished command's result back into the session, and give the
    /// session the turn that its result asks for.
    fn finish_command(
        &mut self,
        started: StartedAttachCommand,
        result: io::Result<command::CommandResult>,
    ) -> io::Result<Option<AttachFinishReason>> {
        let mut result = result?;
        tracing::debug!(exit = result.exit, "attached-client command completed");
        self.background_commands
            .append(&mut result.background_commands);
        self.session
            .complete_command(started.continuation, result, &self.state);
        self.drive(AttachWaitReady::default())
    }
}

/// One source of the session's, named by the role it plays rather than by the
/// descriptor behind it, which the session may replace at any time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachRuntimeSource {
    Input,
    TtyOutput,
    Output,
    Prompt,
    Render,
    Status,
    PopupRead,
    PopupWrite,
}

/// One registered descriptor, with the identity that says whether it is still
/// the one the session means.
struct Registered {
    fd: RawFd,
    generation: u64,
    ready: AsyncFd,
}

/// The descriptors an attached client waits on, owned by its task.
///
/// The session names them by role each time it prepares a wait: a role it does
/// not want is dropped, which is what gives the registration back, and a role
/// whose descriptor was replaced is registered afresh.
#[derive(Default)]
struct Sources {
    input: Option<Registered>,
    tty_output: Option<Registered>,
    output: Option<Registered>,
    prompt: Option<Registered>,
    render: Option<Registered>,
    status: Option<Registered>,
    popup_read: Option<Registered>,
    popup_write: Option<Registered>,
}

impl Sources {
    fn sync(&mut self, runtime: &ClientRuntime, named: &AttachWaitSources) {
        let read = Interest::READABLE;
        let write = Interest::WRITABLE;
        // The pane subscription is the one source that can be replaced without
        // the descriptor number changing, so it carries a generation of its own.
        update(
            &mut self.output,
            runtime,
            named.output,
            named.output_generation,
            read,
        );
        update(&mut self.input, runtime, named.input, 0, read);
        update(&mut self.tty_output, runtime, named.tty_output, 0, write);
        update(&mut self.prompt, runtime, named.prompt, 0, read);
        update(&mut self.render, runtime, named.render, 0, read);
        update(&mut self.status, runtime, named.status, 0, read);
        update(&mut self.popup_read, runtime, named.popup_read, 0, read);
        update(&mut self.popup_write, runtime, named.popup_write, 0, write);
    }

    /// Give every registration back: a client whose session is parked behind a
    /// command it cannot answer waits on the command alone.
    fn clear(&mut self) {
        *self = Self::default();
    }

    /// The wait's arms, in the order a turn where several are ready reports
    /// them.
    fn arms(&self) -> Vec<(AttachRuntimeSource, impl std::future::Future + '_)> {
        [
            (AttachRuntimeSource::Input, self.input.as_ref()),
            (AttachRuntimeSource::TtyOutput, self.tty_output.as_ref()),
            (AttachRuntimeSource::Output, self.output.as_ref()),
            (AttachRuntimeSource::Prompt, self.prompt.as_ref()),
            (AttachRuntimeSource::Render, self.render.as_ref()),
            (AttachRuntimeSource::Status, self.status.as_ref()),
            (AttachRuntimeSource::PopupRead, self.popup_read.as_ref()),
            (AttachRuntimeSource::PopupWrite, self.popup_write.as_ref()),
        ]
        .into_iter()
        .filter_map(|(source, registered)| Some((source, registered?.ready.readiness())))
        .collect()
    }
}

fn update(
    slot: &mut Option<Registered>,
    runtime: &ClientRuntime,
    fd: RawFd,
    generation: u64,
    interest: Interest,
) {
    if fd < 0 {
        *slot = None;
        return;
    }
    if slot
        .as_ref()
        .is_some_and(|registered| registered.fd == fd && registered.generation == generation)
    {
        return;
    }
    // The session owns the descriptor for the length of this call, and the
    // registration takes a duplicate of its own.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    *slot = AsyncFd::new(&runtime.tasks, borrowed, interest)
        .ok()
        .map(|ready| Registered {
            fd,
            generation,
            ready,
        });
}

fn mark_ready(ready: &mut AttachWaitReady, source: AttachRuntimeSource) {
    match source {
        // The tty is drained by the turn that follows, which needs no flag.
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

/// Serve one interactive attach client to its end.
pub(super) async fn run(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: Vec<String>,
    tty: ClientTty,
    context: ClientContext,
) -> ProtocolCloseReason {
    let mut attach = match EventAttachClient::new(
        &args,
        tty,
        Rc::clone(&runtime.state),
        runtime.hub.clone(),
        &context,
        runtime.tasks.clone(),
    ) {
        Ok(attach) => attach,
        // An attach that cannot start is an ordinary command failure: the
        // client is told why over the response it was already waiting for.
        Err(error) => {
            return super::command::report(wire, command::CommandResult::err(error.into_message()))
                .await;
        }
    };
    match serve(wire, runtime, &mut attach).await {
        Ok(reason) | Err(reason) => reason,
    }
}

async fn serve(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    let reason = run_session(wire, runtime, attach).await?;
    finish(wire, runtime, attach, reason).await
}

/// What one wait reported.
enum Wake {
    /// Frames were taken off the socket into the engine's queue.
    Frames,
    /// What the running command is parked on changed.
    Status,
    /// One of the session's sources, or its deadline.
    Ready(AttachWaitReady),
    /// The running command finished.
    Command(io::Result<command::CommandResult>),
}

/// Run the session until it has nothing left to serve, and report why.
async fn run_session(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
) -> Result<AttachFinishReason, ProtocolCloseReason> {
    let mut sources = Sources::default();
    let notify = Notify::new();
    let wake: WakeFn = {
        let notify = notify.clone();
        Rc::new(move || notify.notify())
    };
    let mut command: Option<StartedAttachCommand> = None;
    let mut immediate = 0usize;

    loop {
        // Commands run one at a time, and the session defers them one per pass:
        // a burst — one command per mouse report in a coalesced read — drains
        // through here rather than behind a wait on the tty.
        if command.is_none() {
            command = attach.start_next_command();
        }
        // A suspension the client itself has to answer leaves the session
        // running behind the command; any other one parks the whole client,
        // because nothing it could do would move the command along.
        let session_runs = match &mut command {
            Some(started) => {
                started.task.set_status_wake(&wake);
                started.task.allows_attach_io()
            }
            None => true,
        };

        let mut deadline = None;
        if session_runs {
            match attach.prepare().map_err(fault)? {
                AttachPrepared::Finish(reason) => return Ok(reason),
                AttachPrepared::Ready(ready) => {
                    if let Some(reason) = attach.drive(ready).map_err(fault)? {
                        return Ok(reason);
                    }
                    publish(wire, runtime, attach)?;
                    // The session had work in hand rather than something to
                    // wait for. It keeps going, but not for ever: a busy client
                    // gives the loop a turn before taking its next one.
                    immediate += 1;
                    if immediate >= IMMEDIATE_TURN_BUDGET {
                        immediate = 0;
                        yield_now().await;
                    }
                    continue;
                }
                AttachPrepared::Wait {
                    sources: named,
                    deadline: due,
                } => {
                    sources.sync(runtime, &named);
                    deadline = due;
                }
            }
        } else {
            sources.clear();
        }
        immediate = 0;
        publish(wire, runtime, attach)?;

        // A frame already decoded is not a socket event, so nothing would ever
        // report it: it is taken ahead of the wait rather than waited for.
        if read_frames(wire, attach, session_runs)? {
            continue;
        }

        let woken = wait(
            wire,
            runtime,
            attach,
            &sources,
            deadline,
            session_runs,
            &mut command,
            &notify,
        )
        .await?;
        match woken {
            // Both are the loop looking again with what it now knows.
            Wake::Frames | Wake::Status => {}
            Wake::Ready(ready) => {
                if let Some(reason) = attach.drive(ready).map_err(fault)? {
                    return Ok(reason);
                }
            }
            Wake::Command(result) => {
                let started = command
                    .take()
                    .expect("a command reported with none running");
                if let Some(reason) = attach.finish_command(started, result).map_err(fault)? {
                    return Ok(reason);
                }
            }
        }
        publish(wire, runtime, attach)?;
        // One event per turn: what a wait reported goes behind whatever else
        // the loop already has queued, rather than ahead of it.
        yield_now().await;
    }
}

/// Park until the client, one of the session's sources, its deadline or its
/// running command has something for it.
#[allow(clippy::too_many_arguments)]
async fn wait(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
    sources: &Sources,
    deadline: Option<Instant>,
    reading: bool,
    command: &mut Option<StartedAttachCommand>,
    notify: &Notify,
) -> Result<Wake, ProtocolCloseReason> {
    loop {
        let task = maybe(command.as_mut().map(|started| &mut started.task));
        let timer = maybe(deadline.map(|deadline| sleep_until(&runtime.tasks, deadline)));
        let fds = race(sources.arms());
        // Reading is paused both when the engine has taken all the input it can
        // hold and when this client's own output is above high water; either
        // way the socket is left unread until it is not.
        let readable = maybe(
            (reading && attach.accepts_frames() && wire.is_below_high_water())
                .then(|| wire.readable()),
        );
        let woken = select(
            select(task, notify.notified()),
            select(select(timer, fds), select(readable, wire.writable())),
        )
        .await;
        match woken {
            Either::First(Either::First(result)) => return Ok(Wake::Command(result)),
            Either::First(Either::Second(())) => return Ok(Wake::Status),
            Either::Second(Either::First(Either::First(()))) => {
                return Ok(Wake::Ready(AttachWaitReady::default()))
            }
            Either::Second(Either::First(Either::Second((source, _)))) => {
                let mut ready = AttachWaitReady::default();
                mark_ready(&mut ready, source);
                return Ok(Wake::Ready(ready));
            }
            Either::Second(Either::Second(Either::First(()))) => {
                if read_frames(wire, attach, reading)? {
                    return Ok(Wake::Frames);
                }
            }
            // The socket taking more is not the engine's business; it only
            // means the frames it has queued can move, including the ones held
            // back at the high-water mark.
            Either::Second(Either::Second(Either::Second(()))) => pump(wire, attach)?,
        }
    }
}

/// Take what the client has sent into the engine's queue, in a bounded run: a
/// client that pipelines has its frames taken as one turn's work rather than
/// one turn each.
fn read_frames(
    wire: &mut Wire,
    attach: &mut EventAttachClient,
    reading: bool,
) -> Result<bool, ProtocolCloseReason> {
    if !reading {
        return Ok(false);
    }
    let mut taken = false;
    for _ in 0..READ_FRAME_BUDGET {
        if !attach.accepts_frames() || !wire.is_below_high_water() {
            break;
        }
        let Some(frame) = wire.try_recv()? else {
            break;
        };
        attach.push_frame(frame);
        taken = true;
    }
    Ok(taken)
}

/// Take the terminal back and report the end to the client.
///
/// This was three states of a machine the loop stepped through; it is the same
/// three steps, written in the order they happen: what is queued for the
/// terminal goes out, the terminal is handed back the way it was found, and the
/// client is told how its attach ended.
async fn finish(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
    reason: AttachFinishReason,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    attach.session.stop_terminal();
    let tty = AsyncFd::new(
        &runtime.tasks,
        attach.session.tty_render_fd(),
        Interest::WRITABLE,
    )
    .map_err(fault)?;
    loop {
        attach.session.flush_tty_output().map_err(fault)?;
        if !attach.session.tty_output_pending() {
            break;
        }
        tty.readiness().await;
    }
    attach.session.restore_tty();

    let Some(message) = attach.session.finish_message(reason, &attach.state) else {
        // The client is the one that went away; there is nobody left to tell.
        pump(wire, attach)?;
        wire.drain().await?;
        return Ok(ProtocolCloseReason::Completed);
    };
    attach.output.send(Frame::new(message)).map_err(fault)?;
    pump(wire, attach)?;

    await_finish_ack(wire, runtime, attach).await?;
    attach
        .output
        .send(Frame::new(Message::Exited))
        .map_err(fault)?;
    pump(wire, attach)?;
    wire.drain().await?;
    Ok(ProtocolCloseReason::Completed)
}

/// Wait for the client to acknowledge what it was told — but not for long: a
/// client that has stopped listening must not hold the server here.
async fn await_finish_ack(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
) -> Result<(), ProtocolCloseReason> {
    let deadline = Instant::now() + FINISH_ACK_TIMEOUT;
    loop {
        // Frames taken off the socket before the finish began are the first
        // place the answer can be.
        if let Some(frame) = attach.input.pop() {
            if is_finish_ack(&frame) {
                return Ok(());
            }
            continue;
        }
        match select(wire.recv(), sleep_until(&runtime.tasks, deadline)).await {
            Either::First(Ok(frame)) if is_finish_ack(&frame) => return Ok(()),
            Either::First(Ok(_)) => {}
            // A peer that has stopped talking has said all it is going to.
            Either::First(Err(_)) => return Ok(()),
            Either::Second(()) => return Ok(()),
        }
    }
}

fn is_finish_ack(frame: &Frame) -> bool {
    matches!(frame.msg, Message::Exiting | Message::Exit(..))
}

/// Move what the engine has produced onto the socket, and start what it asked
/// the server to run for it.
fn publish(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    attach: &mut EventAttachClient,
) -> Result<(), ProtocolCloseReason> {
    pump(wire, attach)?;
    for request in attach.take_background_commands() {
        runtime.background.start(request);
    }
    // Whatever this client's commands raised is on the server-wide queue; a
    // publish is the point its own work is done, so the queue gets its turn.
    runtime.background.pump();
    Ok(())
}

/// Move whatever the engine has produced onto the socket, up to what the
/// connection's queue will hold.
fn pump(wire: &mut Wire, attach: &mut EventAttachClient) -> Result<(), ProtocolCloseReason> {
    while wire.is_below_high_water() {
        let Some(frame) = attach.pop_frame() else {
            break;
        };
        wire.queue(frame)?;
    }
    wire.flush()
}

fn fault(error: io::Error) -> ProtocolCloseReason {
    ProtocolCloseReason::Error(error.kind())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

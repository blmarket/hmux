//! The control-mode client: the native control engine, driven as a task.
//!
//! Waiting is this task's own. The descriptors the client reads and writes are
//! registered here for its life and expressed as the arms of one `select`, the
//! deadline is a sleep that disarms when it loses, and a command that has to
//! wait is a future awaited where it was started. What used to be a phase the
//! engine reported to a stepping owner — running, waiting, draining — is the
//! shape of the code below: the queue is parked behind an `await`, and draining
//! is what happens after the loop.

use std::collections::BTreeMap;
use std::io;
use std::os::fd::{AsRawFd, RawFd};
use std::rc::Rc;
use std::time::Instant;

use crate::server::attach::ClientTty;
use crate::server::command::ClientContext;
use crate::server::control::{ControlServing, EventControlClient, StartedControlCommand};
use crate::sync::{maybe, race, select, yield_now, Either};
use hmux_rt::{sleep_until, AsyncFd, Interest};

use super::wire::Wire;
use super::{ClientRuntime, ProtocolCloseReason};

/// Serve one control-mode client to its end.
pub(super) async fn run(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: Vec<String>,
    tty: ClientTty,
    context: ClientContext,
) -> ProtocolCloseReason {
    let mut control = match EventControlClient::new(
        &args,
        tty,
        Rc::clone(&runtime.state),
        runtime.hub.clone(),
        &context,
        runtime.tasks.clone(),
    ) {
        Ok(control) => control,
        Err(error) => return ProtocolCloseReason::Error(error.kind()),
    };
    // Control mode moves subsequent input to the passed stdin fd, so the socket
    // is write-only from here: the native control loop likewise stops reading
    // imsg once it has identified.
    match serve(wire, runtime, &mut control).await {
        Ok(reason) | Err(reason) => reason,
    }
}

/// What one wait reported.
#[derive(Clone, Copy)]
enum Ready {
    Input,
    Output,
    State,
    Pane(u32),
    Timer,
}

/// The descriptors one control client waits on, owned by its task.
///
/// The client's own descriptors live as long as it does, so they are registered
/// once; a source it is not interested in right now is an arm left out of the
/// wait, not a registration handed back. Pane subscriptions are the exception:
/// they come and go with the panes, so they are reconciled before each wait.
struct Sources {
    input: Option<AsyncFd>,
    output: AsyncFd,
    state: AsyncFd,
    panes: BTreeMap<u32, (RawFd, AsyncFd)>,
}

impl Sources {
    fn new(runtime: &ClientRuntime, control: &EventControlClient) -> io::Result<Self> {
        let input = match control.stdin_fd() {
            Some(fd) => Some(AsyncFd::new(&runtime.tasks, fd, Interest::READABLE)?),
            None => None,
        };
        Ok(Self {
            input,
            output: AsyncFd::new(&runtime.tasks, control.output_fd(), Interest::WRITABLE)?,
            state: AsyncFd::new(&runtime.tasks, control.state_fd(), Interest::READABLE)?,
            panes: BTreeMap::new(),
        })
    }

    /// Bring the pane registrations in line with the streams the client has.
    ///
    /// A pane that went away has its registration dropped, which is what gives
    /// it back; a pane whose subscription was replaced is registered afresh,
    /// since the notification comes on the new descriptor.
    fn sync_panes(&mut self, runtime: &ClientRuntime, control: &EventControlClient) {
        let wanted = control.pane_fds();
        self.panes.retain(|pane_id, (raw, _)| {
            wanted
                .iter()
                .any(|(id, fd)| id == pane_id && fd.as_raw_fd() == *raw)
        });
        for (pane_id, fd) in wanted {
            if self.panes.contains_key(&pane_id) {
                continue;
            }
            if let Ok(registered) = AsyncFd::new(&runtime.tasks, fd, Interest::READABLE) {
                self.panes.insert(pane_id, (fd.as_raw_fd(), registered));
            }
        }
    }

    /// The read side of the client's input, armed only while it still has input
    /// to give: a descriptor at end of file is ready forever.
    fn input(&self, control: &EventControlClient) -> impl std::future::Future<Output = ()> + '_ {
        let armed = control.stdin_open();
        let input = self.input.as_ref();
        async move {
            match input.filter(|_| armed) {
                Some(input) => {
                    input.readiness().await;
                }
                None => std::future::pending().await,
            }
        }
    }

    /// The write side of the client's output, armed only while bytes are queued
    /// for it.
    fn output(&self, control: &EventControlClient) -> impl std::future::Future<Output = ()> + '_ {
        let armed = control.output_pending();
        let output = &self.output;
        async move {
            if !armed {
                std::future::pending::<()>().await;
            }
            output.readiness().await;
        }
    }
}

async fn serve(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    let mut sources = Sources::new(runtime, control).map_err(fault)?;
    let mut ready: Option<Ready> = None;

    'serve: loop {
        if control.refresh().map_err(fault)? == ControlServing::Stop {
            break;
        }

        let mut state_ready = false;
        let mut pane_ready = false;
        let mut input_pending = false;
        match ready.take() {
            Some(Ready::Input) => input_pending = control.read_input().map_err(fault)?,
            Some(Ready::Output) => control.flush_output().map_err(fault)?,
            Some(Ready::State) => {
                control.note_state_change();
                state_ready = true;
            }
            Some(Ready::Pane(pane_id)) => pane_ready = control.note_pane_output(pane_id),
            Some(Ready::Timer) | None => {}
        }
        control.parse_input().map_err(fault)?;

        // Commands run one at a time, and the rest of the queue is parked
        // behind whichever one is running: that wait is the await below.
        while let Some(started) = control.start_next_command().map_err(fault)? {
            if run_command(wire, runtime, control, &sources, started).await? == ControlServing::Stop
            {
                break 'serve;
            }
        }

        let serving = control
            .finish_turn(state_ready, pane_ready)
            .map_err(fault)?;
        publish(wire, runtime, control)?;
        if serving == ControlServing::Stop {
            break;
        }

        if input_pending {
            // Work the client already has takes its turn behind whatever else
            // the loop has queued, the granularity one event per dispatch had.
            ready = Some(Ready::Input);
            yield_now().await;
            continue;
        }
        ready = Some(wait(wire, runtime, control, &mut sources).await?);
        // One event per turn: what a wait reported goes behind whatever else
        // the loop already has queued, rather than ahead of it.
        yield_now().await;
    }

    drain(wire, control, &sources).await
}

/// Await one started command, keeping the client's socket moving meanwhile.
///
/// Nothing else the client has queued starts here — the queue is parked behind
/// this command. What does keep going is what the client did while parked
/// before: input read into its parse buffer, output handed to its descriptor,
/// and actions another client aimed at it, one of which can end it.
async fn run_command(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
    sources: &Sources,
    mut started: StartedControlCommand,
) -> Result<ControlServing, ProtocolCloseReason> {
    let result = loop {
        control.flush_output().map_err(fault)?;
        publish(wire, runtime, control)?;

        let woken = select(
            select(&mut started.task, sources.input(control)),
            select(sources.output(control), wire.writable()),
        )
        .await;
        match woken {
            Either::First(Either::First(result)) => break result,
            Either::First(Either::Second(())) => {
                // Readiness is edge-triggered, so a read that runs out of
                // budget with the descriptor still readable comes back for the
                // rest rather than waiting for an edge that never comes.
                while control.read_input().map_err(fault)? {
                    yield_now().await;
                }
            }
            Either::Second(Either::First(())) => control.flush_output().map_err(fault)?,
            // The socket taking more is not the command's business; it only
            // means the frames already queued can move.
            Either::Second(Either::Second(())) => pump(wire, control)?,
        }
        // Between waits the client is serviced as it was while parked: what it
        // typed is parsed for the queue behind this command, and an action
        // another client aimed at it can end it. The command's own turn comes
        // first, so what it reports precedes what anyone else asked for.
        if control.refresh().map_err(fault)? == ControlServing::Stop {
            return Ok(ControlServing::Stop);
        }
        control.parse_input().map_err(fault)?;
    };
    control.finish_command(started, result).map_err(fault)?;
    Ok(ControlServing::Continue)
}

/// Park until one of the client's descriptors, its deadline, or the socket has
/// something to say, flushing the socket meanwhile.
async fn wait(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
    sources: &mut Sources,
) -> Result<Ready, ProtocolCloseReason> {
    sources.sync_panes(runtime, control);
    let deadline = control.deadline(Instant::now());
    loop {
        let panes = race(
            sources
                .panes
                .iter()
                .map(|(pane_id, (_, fd))| (*pane_id, fd.readiness()))
                .collect(),
        );
        let timer = maybe(deadline.map(|deadline| sleep_until(&runtime.tasks, deadline)));
        let woken = select(
            select(timer, sources.input(control)),
            select(
                select(sources.output(control), sources.state.readiness()),
                select(panes, wire.writable()),
            ),
        )
        .await;
        match woken {
            Either::First(Either::First(())) => return Ok(Ready::Timer),
            Either::First(Either::Second(())) => return Ok(Ready::Input),
            Either::Second(Either::First(Either::First(()))) => return Ok(Ready::Output),
            Either::Second(Either::First(Either::Second(_))) => return Ok(Ready::State),
            Either::Second(Either::Second(Either::First((pane_id, _)))) => {
                return Ok(Ready::Pane(pane_id))
            }
            // The socket taking more is not the engine's business; it only
            // means the frames it has queued can move, including the ones held
            // back at the high-water mark.
            Either::Second(Either::Second(Either::Second(()))) => pump(wire, control)?,
        }
    }
}

/// Push out everything the client has queued, then report its exit.
///
/// This is where a control client ends, whichever way it got here: its own
/// input closed, a `detach-client` aimed at it, or its session going away.
async fn drain(
    wire: &mut Wire,
    control: &mut EventControlClient,
    sources: &Sources,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    loop {
        control.flush_output().map_err(fault)?;
        if !control.output_pending() {
            break;
        }
        sources.output.readiness().await;
    }
    while let Some(frame) = control.pop_frame() {
        wire.send(frame).await?;
    }
    wire.send(control.exit_frame()).await?;
    wire.drain().await?;
    Ok(ProtocolCloseReason::Completed)
}

/// Move what the engine has produced onto the socket, and start what it asked
/// the server to run for it.
fn publish(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
) -> Result<(), ProtocolCloseReason> {
    pump(wire, control)?;
    for request in control.take_background_commands() {
        runtime.background.start(request);
    }
    Ok(())
}

/// Move whatever the engine has produced onto the socket, up to what the
/// connection's queue will hold.
fn pump(wire: &mut Wire, control: &mut EventControlClient) -> Result<(), ProtocolCloseReason> {
    while wire.is_below_high_water() {
        let Some(frame) = control.pop_frame() else {
            break;
        };
        wire.queue(frame)?;
    }
    wire.flush()
}

fn fault(error: io::Error) -> ProtocolCloseReason {
    ProtocolCloseReason::Error(error.kind())
}

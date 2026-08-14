//! The control-mode client: the native control engine, driven as a task.
//!
//! The engine reports what it is waiting on and is handed back whichever of
//! those became ready. What used to describe that set to a central loop — an
//! interest diff, a generation-keyed timer, a wake routed by source — is now
//! the wait itself: the descriptors are the task's own registrations, the
//! deadline is a sleep that disarms when it loses, and the parked command
//! queue is a notification.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::time::Instant;

use crate::server::attach::ClientTty;
use crate::server::command::ClientContext;
use crate::server::control::{EventControlClient, EventControlSource};
use crate::sync::{maybe, race, select, yield_now, Either, Notify, WakeFn};
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
        Rc::clone(&runtime.commands),
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
enum Wake {
    /// The parked command queue moved, named by the source it was parked as.
    Command(Option<EventControlSource>),
    /// A deadline the engine asked for landed.
    Timer,
    /// One of the engine's descriptors is ready.
    Source(EventControlSource),
}

async fn serve(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    let notify = Notify::new();
    let wake: WakeFn = {
        let notify = notify.clone();
        Rc::new(move || notify.notify())
    };
    let mut sources: BTreeMap<EventControlSource, AsyncFd> = BTreeMap::new();
    let mut ready: Option<EventControlSource> = None;

    loop {
        control
            .drive(ready)
            .map_err(|error| ProtocolCloseReason::Error(error.kind()))?;

        pump(wire, control)?;
        for request in control.take_background_commands() {
            runtime.background.start(request);
        }

        if control.is_finished() {
            wire.drain().await?;
            return Ok(ProtocolCloseReason::Completed);
        }

        // Work the engine already has: it takes its turn behind whatever else
        // the loop has queued, the granularity one event per dispatch had.
        if control.take_input_continuation() {
            ready = Some(EventControlSource::Input);
            yield_now().await;
            continue;
        }
        if control.take_command_continuation() {
            ready = None;
            yield_now().await;
            continue;
        }

        ready = match wait(wire, runtime, control, &mut sources, &notify, &wake).await? {
            Wake::Command(source) => source,
            Wake::Timer => None,
            Wake::Source(source) => Some(source),
        };
        // One event per turn: what a wait reported goes behind whatever else
        // the loop already has queued, rather than ahead of it.
        yield_now().await;
    }
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

/// Park until one of the engine's sources, its deadline, or its parked command
/// has something to say, flushing the socket meanwhile.
async fn wait(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    control: &mut EventControlClient,
    sources: &mut BTreeMap<EventControlSource, AsyncFd>,
    notify: &Notify,
    wake: &WakeFn,
) -> Result<Wake, ProtocolCloseReason> {
    let desired: BTreeSet<EventControlSource> = control.sources().into_iter().collect();
    // A registration the engine no longer wants goes back by being dropped.
    sources.retain(|source, _| desired.contains(source));
    let mut command = None;
    for source in &desired {
        if let EventControlSource::Command(generation) = source {
            // No descriptor behind a suspended queue: what it offers instead is
            // the wake its completion fires, which this turns into one
            // notification the wait can race.
            control.set_command_wake(*generation, wake);
            command = Some(*source);
            continue;
        }
        if sources.contains_key(source) {
            continue;
        }
        // A source whose descriptor is already gone is dropped the way a
        // vanished registration is.
        let Some(fd) = control.source_fd(*source) else {
            continue;
        };
        let interest = if EventControlClient::source_is_writable(*source) {
            Interest::WRITABLE
        } else {
            Interest::READABLE
        };
        if let Ok(registered) = AsyncFd::new(&runtime.tasks, fd, interest) {
            sources.insert(*source, registered);
        }
    }
    let deadline = control.deadline(Instant::now());

    loop {
        let readiness = race(
            sources
                .iter()
                .map(|(source, fd)| (*source, fd.readiness()))
                .collect(),
        );
        let timer = maybe(deadline.map(|deadline| sleep_until(&runtime.tasks, deadline)));
        let woken = select(
            select(notify.notified(), timer),
            select(readiness, wire.writable()),
        )
        .await;
        match woken {
            Either::First(Either::First(())) => return Ok(Wake::Command(command)),
            Either::First(Either::Second(())) => return Ok(Wake::Timer),
            Either::Second(Either::First((source, _))) => return Ok(Wake::Source(source)),
            // The socket taking more is not the engine's business; it only
            // means the frames it has queued can move, including the ones held
            // back at the high-water mark.
            Either::Second(Either::Second(())) => pump(wire, control)?,
        }
    }
}

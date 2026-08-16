//! Task-driven pane PTY I/O.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::server::pane::PaneIo;

use crate::sync::{maybe, select, yield_now, Either, Notify};
use hmux_rt::{sleep_until, AsyncFd, Interest, JoinHandle, Readiness, TaskHandle};

/// tmux's ground timer: how long `input.c` waits for the terminator of a
/// string sequence before giving up on it and returning the parser to ground,
/// so a pane that opens an OSC and never closes it does not swallow everything
/// it writes afterwards.
const GROUND_TIMEOUT: Duration = Duration::from_secs(5);

/// What sends the task to the read half. Either end closing, and an error,
/// join readable here: each of them is only observed by reading, which returns
/// the end-of-file or the failure that ends the pane.
const DRAINABLE: Readiness = Readiness::READABLE
    .or(Readiness::READ_CLOSED)
    .or(Readiness::WRITE_CLOSED)
    .or(Readiness::ERROR);

/// The loop's handle to one pane task.
pub(crate) struct PaneHandle {
    task: JoinHandle<()>,
    poke: Notify,
    write_pending: Rc<dyn Fn() -> bool>,
    /// Whether a poke is already out for the current spell of queued input.
    /// The task clears it when the queue drains; while a full PTY holds the
    /// input back — a child ignoring its stdin — this stays set, so the host's
    /// sync passes stop producing wakes and the loop can block in the reactor.
    poke_armed: Rc<Cell<bool>>,
}

impl PaneHandle {
    pub(crate) fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }

    /// Ask the task to stop; it finishes on its next turn.
    pub(crate) fn shutdown(&self) {
        self.task.cancel();
    }

    /// Wake the pane task when the server has queued input for its PTY.
    ///
    /// The task writes opportunistically on the poke: the PTY is almost always
    /// writable, and after a real `WouldBlock` the kernel owes it a writable
    /// edge, so nothing re-registers interest.
    pub(crate) fn poke_write(&self) {
        if (self.write_pending)() && !self.poke_armed.get() {
            self.poke_armed.set(true);
            self.poke.notify();
        }
    }
}

impl Drop for PaneHandle {
    fn drop(&mut self) {
        self.task.cancel();
    }
}

enum Wakeup {
    Io(Readiness),
    Poke,
    Ground,
    /// A control client that had fallen behind this pane's output caught up,
    /// so the reads held back for it can go on.
    Resume,
}

/// Drive one pane's PTY on the loop.
pub(crate) fn spawn(tasks: &TaskHandle, mut io: PaneIo) -> PaneHandle {
    let poke = Notify::new();
    let poke_armed = Rc::new(Cell::new(false));
    let write_pending = io.write_probe();
    let task_poke = poke.clone();
    let task_armed = Rc::clone(&poke_armed);
    let handle = tasks.clone();
    let task = tasks.spawn_join(async move {
        run(&handle, &mut io, &task_poke, &task_armed).await;
    });
    PaneHandle {
        task,
        poke,
        write_pending,
        poke_armed,
    }
}

async fn run(tasks: &TaskHandle, io: &mut PaneIo, poke: &Notify, poke_armed: &Cell<bool>) {
    let Ok(readiness) = AsyncFd::new(tasks, io.as_fd(), Interest::READABLE | Interest::WRITABLE)
    else {
        return;
    };
    // Whether the parser was inside a string sequence at the end of the last
    // read. tmux restarts the timer whenever such a sequence *begins*; this
    // watches the same edge once per read, which against five seconds is not
    // an observable difference.
    let mut awaiting = false;
    let mut ground_deadline: Option<Instant> = None;
    let resume = io.output_resume();
    loop {
        let ground = maybe(ground_deadline.map(|deadline| sleep_until(tasks, deadline)));
        let wakeup = match select(
            select(readiness.readiness(), poke.notified()),
            select(ground, resume.notified()),
        )
        .await
        {
            Either::First(Either::First(ready)) => Wakeup::Io(ready),
            Either::First(Either::Second(())) => Wakeup::Poke,
            Either::Second(Either::First(())) => Wakeup::Ground,
            Either::Second(Either::Second(())) => Wakeup::Resume,
        };
        match wakeup {
            Wakeup::Ground => {
                ground_deadline = None;
                awaiting = false;
                io.expire_ground();
                continue;
            }
            Wakeup::Poke => {
                io.drive_writable();
                poke_armed.set(io.wants_write());
            }
            // A read held back for a lagging control client resumes here
            // rather than on readiness: the descriptor stayed readable while
            // it waited, so its edge is long past.
            Wakeup::Resume => {
                if drain_reads(io, &mut awaiting, &mut ground_deadline).await == Draining::Ended {
                    return;
                }
            }
            Wakeup::Io(ready) => {
                if ready.is_writable() {
                    io.drive_writable();
                    poke_armed.set(io.wants_write());
                }
                if ready.intersects(DRAINABLE)
                    && drain_reads(io, &mut awaiting, &mut ground_deadline).await == Draining::Ended
                {
                    return;
                }
            }
        }
        sync_ground_timer(io, &mut awaiting, &mut ground_deadline);
    }
}

/// Whether the pane still has a PTY to read.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Draining {
    Continues,
    Ended,
}

/// Read until the PTY has nothing more, or until a control client's backlog
/// says the rest has to wait.
///
/// Edge-style: read until `WouldBlock` before waiting again, yielding after
/// every coalesced chunk so a pane flood cannot starve the rest of the loop.
async fn drain_reads(
    io: &mut PaneIo,
    awaiting: &mut bool,
    ground_deadline: &mut Option<Instant>,
) -> Draining {
    loop {
        // Checked before the read, not after: what has been read is already in
        // the journal, and reading more of a flood a control client has not
        // taken yet is what used to discard it.
        if io.output_backpressured() {
            return Draining::Continues;
        }
        let result = match io.drive_readable() {
            Ok(result) => result,
            Err(_) => return Draining::Ended,
        };
        if result.closed {
            return Draining::Ended;
        }
        sync_ground_timer(io, awaiting, ground_deadline);
        // The timer keeps its place inside a burst: the actor dispatched its
        // expiry between read continuations, and a task checks the deadline in
        // the same seams.
        if expire_if_due(io, awaiting, ground_deadline) {
            continue;
        }
        if !result.continuation {
            return Draining::Continues;
        }
        yield_now().await;
    }
}

/// Arm the ground timer when the pane has just entered a string sequence,
/// and drop it when the sequence is over. `input_start_ground_timer` runs
/// on the way into each of those states and `input_ground` cancels it, so
/// the timer measures from where the sequence began rather than from the
/// last byte of it.
fn sync_ground_timer(io: &PaneIo, awaiting: &mut bool, ground_deadline: &mut Option<Instant>) {
    let now_awaiting = io.awaiting_terminator();
    if now_awaiting == *awaiting {
        return;
    }
    *awaiting = now_awaiting;
    *ground_deadline = now_awaiting.then(|| Instant::now() + GROUND_TIMEOUT);
}

/// Expire a sequence whose armed deadline has already passed.
fn expire_if_due(io: &PaneIo, awaiting: &mut bool, ground_deadline: &mut Option<Instant>) -> bool {
    let due = ground_deadline.is_some_and(|deadline| Instant::now() >= deadline);
    if due {
        *ground_deadline = None;
        *awaiting = false;
        io.expire_ground();
    }
    due
}

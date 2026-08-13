//! Host-side driving of one task set.
//!
//! Registration sync, deadline timers, and readiness delivery are the same
//! plumbing in every host that embeds a [`TaskSet`], so they live here once.
//! The host keeps what makes it a host: the reactor, the event queue, and the
//! dispatch order. Everything the loop produces — a spawned task's first poll,
//! an elapsed deadline — is reported through the callbacks the host passes in,
//! so the host queues it in its own order.

use std::collections::HashMap;
use std::io;
use std::os::fd::AsFd as _;
use std::time::{Duration, Instant};

use crate::reactor::{Reactor, Readiness};
use crate::tasks::{TaskEvent, TaskHandle, TaskId, TaskSet, WakeSink};
use crate::timer::{ExpiredTimer, TimerId, TimerQueue};

/// One task set with the timer and registration bookkeeping its host owes it.
pub struct TaskLoop {
    tasks: TaskSet,
    handle: TaskHandle,
    /// The timer armed for each sleeping task, at most one apiece.
    timers: TimerQueue<TaskId>,
    armed: HashMap<TaskId, (Instant, TimerId)>,
    expired: Vec<ExpiredTimer<TaskId>>,
}

impl TaskLoop {
    /// Wakes are reported through `wake`; the host queues "poll this task"
    /// however it orders the rest of its work.
    pub fn new(wake: WakeSink) -> Self {
        let (tasks, handle) = TaskSet::new(wake);
        Self {
            tasks,
            handle,
            timers: TimerQueue::new(),
            armed: HashMap::new(),
            expired: Vec::new(),
        }
    }

    pub fn handle(&self) -> TaskHandle {
        self.handle.clone()
    }

    /// Adopt newly spawned tasks and make the reactor and the timer queue
    /// describe what the live ones are waiting for.
    ///
    /// Unlike effect-driven interest updates there is nothing to diff: a
    /// task's descriptors are owned by the leaves that created them, so this
    /// only makes registrations that were asked for and releases the ones
    /// whose `AsyncFd` is gone. `recipient` names a task registration in the
    /// host's own readiness address space; readiness delivered to it comes
    /// back through [`TaskLoop::deliver_io`] with the same id.
    pub fn sync<R, T>(
        &mut self,
        reactor: &mut R,
        recipient: impl Fn(u64) -> T,
        mut enqueue: impl FnMut(TaskEvent),
    ) -> io::Result<()>
    where
        R: Reactor<T>,
        T: Clone,
    {
        for token in self.tasks.take_released_io() {
            reactor.deregister(token)?;
        }
        for (io, task, fd, interest) in self.tasks.take_new_io() {
            match reactor.register(fd.as_fd(), interest, recipient(io)) {
                Ok(token) => self.tasks.set_io_token(io, token),
                // A descriptor with no poll operation — a regular file, or a
                // client that redirected its output to `/dev/null` — is
                // rejected by `epoll_ctl` with EPERM. Such a descriptor is
                // never *not* ready, so serve it directly instead of taking
                // the host down with it.
                Err(error) if error.raw_os_error() == Some(libc::EPERM) => {
                    self.tasks.mark_io_unpollable(io);
                    enqueue(TaskEvent::Poll(task));
                }
                Err(error) => return Err(error),
            }
            // The reactor took its own duplicate.
            drop(fd);
        }
        let deadlines = self.tasks.deadlines();
        self.armed.retain(|task, (_, timer)| {
            let keep = deadlines.contains_key(task);
            if !keep {
                self.timers.cancel(*timer);
            }
            keep
        });
        for (task, deadline) in deadlines {
            match self.armed.get(&task) {
                Some((armed, _)) if *armed == deadline => continue,
                Some((_, timer)) => {
                    self.timers.cancel(*timer);
                }
                None => {}
            }
            let timer = self.timers.set(deadline, task);
            self.armed.insert(task, (deadline, timer));
        }
        // A task's first turn is an event like any other, so a spawn never
        // runs ahead of work already queued.
        for task in self.tasks.take_spawned() {
            enqueue(TaskEvent::Poll(task));
        }
        Ok(())
    }

    /// Poll the task one event names.
    pub fn dispatch(&mut self, event: TaskEvent) {
        match event {
            TaskEvent::Poll(task) | TaskEvent::Timeout(task) => self.tasks.poll(task),
        }
    }

    /// Hand readiness to the descriptor it was registered for, reporting the
    /// task to poll. `None` once the `AsyncFd` is gone.
    pub fn deliver_io(&mut self, io: u64, readiness: Readiness) -> Option<TaskId> {
        self.tasks.deliver_io(io, readiness)
    }

    /// How long the host may block before the nearest task deadline.
    pub fn time_until_next_deadline(&mut self, now: Instant) -> Option<Duration> {
        self.timers.time_until_next(now)
    }

    /// Turn every elapsed deadline into an event.
    pub fn drain_expired(&mut self, now: Instant, mut enqueue: impl FnMut(TaskEvent)) {
        self.timers.drain_expired(now, &mut self.expired);
        for timer in self.expired.drain(..) {
            enqueue(TaskEvent::Timeout(timer.into_value()));
        }
    }

    /// Deadlines currently armed. Zero once every sleeping task has resumed.
    pub fn armed_timers(&self) -> usize {
        self.timers.len()
    }

    /// Tasks spawned since the last sync, still owed their first poll.
    pub fn pending_spawned(&self) -> usize {
        self.tasks.pending_spawned()
    }
}

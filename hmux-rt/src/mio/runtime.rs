use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::os::fd::{OwnedFd, RawFd};
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{ContextBuilder, LocalWake, LocalWaker, Waker};
use std::time::{Duration, Instant};

use super::readiness::IoState;
use super::signals::SignalState;
use super::{Io, Signals, Sleep};

// Bound each turn so a self-waking task cannot monopolize the host thread.
const MAX_POLLS_PER_TURN: usize = 128;

type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "runtime is no longer usable in this process",
    )
}

pub(crate) struct Core {
    pub(crate) pid: u32,
    pub(crate) alive: Cell<bool>,
    poisoned: Cell<bool>,
    driving: Cell<bool>,
    next_id: Cell<usize>,
    pub(crate) registry: RefCell<Option<mio::Registry>>,
    tasks: RefCell<HashMap<usize, Rc<TaskState>>>,
    ready: RefCell<VecDeque<Weak<TaskState>>>,
    pub(crate) timers: RefCell<BTreeMap<(Instant, usize), LocalWaker>>,
    pub(crate) io: RefCell<HashMap<usize, Weak<IoState>>>,
    pub(crate) fds: RefCell<HashMap<RawFd, usize>>,
    pub(crate) signals: RefCell<HashMap<usize, Weak<SignalState>>>,
    pub(crate) error: RefCell<Option<io::Error>>,
}

impl Core {
    pub(crate) fn check(&self) -> io::Result<()> {
        if !self.alive.get() || self.pid != std::process::id() {
            return Err(invalid());
        }
        if self.poisoned.get() {
            return Err(io::Error::other("runtime poisoned by a task panic"));
        }
        Ok(())
    }

    pub(crate) fn allocate(&self) -> io::Result<usize> {
        self.check()?;
        let id = self.next_id.get();
        self.next_id.set(
            id.checked_add(1)
                .ok_or_else(|| io::Error::other("identity space exhausted"))?,
        );
        Ok(id)
    }

    pub(crate) fn record(&self, error: io::Error) {
        let mut pending = self.error.borrow_mut();
        if pending.is_none() {
            *pending = Some(error);
        }
    }

    fn compact_ready(&self) {
        let threshold = self
            .tasks
            .borrow()
            .len()
            .saturating_mul(2)
            .saturating_add(64);
        let mut ready = self.ready.borrow_mut();
        if ready.len() > threshold {
            ready.retain(|weak| weak.upgrade().is_some_and(|task| !task.cancelled.get()));
        }
    }

    fn runnable(&self) -> bool {
        let mut ready = self.ready.borrow_mut();
        while ready
            .front()
            .is_some_and(|weak| weak.upgrade().is_none_or(|task| task.cancelled.get()))
        {
            ready.pop_front();
        }
        !ready.is_empty()
    }

    fn close(&self) {
        if !self.alive.replace(false) {
            return;
        }
        let normal = self.pid == std::process::id();
        let signals = self
            .signals
            .borrow_mut()
            .drain()
            .filter_map(|(_, s)| s.upgrade())
            .collect::<Vec<_>>();
        for signal in signals {
            signal.close();
        }
        let tasks = self
            .tasks
            .borrow_mut()
            .drain()
            .map(|(_, t)| t)
            .collect::<Vec<_>>();
        for task in tasks {
            task.cancel();
        }
        let sources = self
            .io
            .borrow_mut()
            .drain()
            .filter_map(|(_, s)| s.upgrade())
            .collect::<Vec<_>>();
        for source in sources {
            source.close(normal);
        }
        self.registry.borrow_mut().take();
        self.fds.borrow_mut().clear();
        self.timers.borrow_mut().clear();
        self.ready.borrow_mut().clear();
    }
}

struct TaskState {
    core: Weak<Core>,
    id: usize,
    future: RefCell<Option<TaskFuture>>,
    cancelled: Cell<bool>,
    queued: Cell<bool>,
    waker: LocalWaker,
}

impl TaskState {
    fn cancel(&self) {
        if self.cancelled.replace(true) {
            return;
        }
        self.queued.set(false);
        if let Some(core) = self.core.upgrade() {
            core.tasks.borrow_mut().remove(&self.id);
            core.compact_ready();
        }
        // Never run a user destructor while borrowing the task slot/table.
        let future = self.future.borrow_mut().take();
        drop(future);
    }

    fn enqueue(self: &Rc<Self>) {
        let Some(core) = self.core.upgrade() else {
            return;
        };
        if core.check().is_err() || self.cancelled.get() || self.queued.replace(true) {
            return;
        }
        core.ready.borrow_mut().push_back(Rc::downgrade(self));
    }
}

struct TaskWake(Weak<TaskState>);
impl LocalWake for TaskWake {
    fn wake(self: Rc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Rc<Self>) {
        if let Some(task) = self.0.upgrade() {
            task.enqueue();
        }
    }
}

/// Cancel-on-drop owner for a local task. It is neither cloneable nor detachable.
pub struct Task {
    state: Rc<TaskState>,
}

impl Drop for Task {
    fn drop(&mut self) {
        self.state.cancel();
    }
}

/// Cloneable, thread-local capability bound to one runtime generation.
#[derive(Clone)]
pub struct Handle {
    pub(crate) core: Rc<Core>,
}

impl crate::Handle for Handle {
    type Task = Task;
    type Io = Io;
    type Signals = Signals;
    type Sleep = Sleep;

    fn spawn<F>(&self, future: F) -> io::Result<Task>
    where
        F: Future<Output = ()> + 'static,
    {
        let id = self.core.allocate()?;
        let state = Rc::new_cyclic(|weak| TaskState {
            core: Rc::downgrade(&self.core),
            id,
            future: RefCell::new(Some(Box::pin(future))),
            cancelled: Cell::new(false),
            queued: Cell::new(false),
            waker: LocalWaker::from(Rc::new(TaskWake(weak.clone()))),
        });
        self.core.tasks.borrow_mut().insert(id, state.clone());
        state.enqueue();
        Ok(Task { state })
    }

    fn io(&self, fd: Rc<OwnedFd>) -> io::Result<Io> {
        Io::new(&self.core, fd)
    }
    fn signals(&self, set: &[std::ffi::c_int]) -> io::Result<Signals> {
        Signals::new(&self.core, set)
    }
    fn sleep_until(&self, deadline: Instant) -> Sleep {
        Sleep::new(&self.core, deadline)
    }
}

/// Host-driven local-waker executor with a mio Unix readiness backend.
pub struct Runtime {
    core: Rc<Core>,
    poller: mio::Poll,
    events: mio::Events,
}

struct Driving(Rc<Core>);
impl Drop for Driving {
    fn drop(&mut self) {
        self.0.driving.set(false);
    }
}

impl crate::Runtime for Runtime {
    type Handle = Handle;

    fn new() -> io::Result<Self> {
        let poller = mio::Poll::new()?;
        let registry = poller.registry().try_clone()?;
        Ok(Self {
            core: Rc::new(Core {
                pid: std::process::id(),
                alive: Cell::new(true),
                poisoned: Cell::new(false),
                driving: Cell::new(false),
                next_id: Cell::new(1),
                registry: RefCell::new(Some(registry)),
                tasks: RefCell::default(),
                ready: RefCell::default(),
                timers: RefCell::default(),
                io: RefCell::default(),
                fds: RefCell::default(),
                signals: RefCell::default(),
                error: RefCell::default(),
            }),
            poller,
            events: mio::Events::with_capacity(1024),
        })
    }

    fn handle(&self) -> Handle {
        Handle {
            core: self.core.clone(),
        }
    }

    fn poll(&mut self, max_wait: Option<Duration>) -> io::Result<()> {
        self.core.check()?;
        if self.core.driving.replace(true) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "reentrant driving",
            ));
        }
        let _driving = Driving(self.core.clone());
        if let Some(error) = self.core.error.borrow_mut().take() {
            return Err(error);
        }
        self.wake_timers();
        let timeout = if self.core.runnable() || self.core.tasks.borrow().is_empty() {
            Some(Duration::ZERO)
        } else {
            let timer = self
                .core
                .timers
                .borrow()
                .first_key_value()
                .map(|(key, _)| key.0.saturating_duration_since(Instant::now()));
            match (max_wait, timer) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, None) => a,
                (None, b) => b,
            }
        };
        let stop = timeout.and_then(|duration| Instant::now().checked_add(duration));
        loop {
            self.events.clear();
            let remaining = stop.map(|stop| stop.saturating_duration_since(Instant::now()));
            match self.poller.poll(&mut self.events, remaining) {
                Ok(()) => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                    if stop.is_some_and(|stop| stop <= Instant::now()) {
                        break;
                    }
                }
                Err(error) => return Err(error),
            }
        }
        for event in &self.events {
            let source = self
                .core
                .io
                .borrow()
                .get(&event.token().0)
                .and_then(Weak::upgrade);
            if let Some(source) = source {
                source.observe(event)?;
            }
        }
        self.wake_timers();
        let mut polled = 0;
        while polled < MAX_POLLS_PER_TURN {
            let task = self.core.ready.borrow_mut().pop_front();
            let Some(weak) = task else {
                break;
            };
            let Some(task) = weak.upgrade() else {
                continue;
            };
            if task.cancelled.get() {
                continue;
            }
            task.queued.set(false);
            let future = task.future.borrow_mut().take();
            let Some(mut future) = future else {
                continue;
            };
            polled += 1;
            let mut context = ContextBuilder::from_waker(Waker::noop())
                .local_waker(&task.waker)
                .build();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                future.as_mut().poll(&mut context)
            }));
            match result {
                Ok(result) if result.is_pending() && !task.cancelled.get() => {
                    *task.future.borrow_mut() = Some(future);
                }
                Ok(_) => {
                    task.cancel();
                    drop(future);
                }
                Err(panic) => {
                    self.core.poisoned.set(true);
                    task.cancel();
                    drop(future);
                    std::panic::resume_unwind(panic);
                }
            }
        }
        if let Some(error) = self.core.error.borrow_mut().take() {
            return Err(error);
        }
        Ok(())
    }

    fn reset_after_fork(&mut self) -> io::Result<()> {
        if self.core.pid == std::process::id() || self.core.driving.get() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "reset requires an idle inherited runtime",
            ));
        }
        let replacement = Self::new()?;
        *self = replacement;
        Ok(())
    }
}

impl Runtime {
    fn wake_timers(&self) {
        let now = Instant::now();
        loop {
            let wake = {
                let mut timers = self.core.timers.borrow_mut();
                if timers
                    .first_key_value()
                    .is_some_and(|(key, _)| key.0 <= now)
                {
                    timers.pop_first().map(|(_, wake)| wake)
                } else {
                    None
                }
            };
            match wake {
                Some(wake) => wake.wake(),
                None => break,
            }
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.core.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Handle as _, Runtime as _};
    use std::task::{Context, Poll};

    #[test]
    fn cancellation_storage_is_bounded_without_a_driver_turn() {
        let runtime = Runtime::new().unwrap();
        let handle = runtime.handle();
        for _ in 0..10_000 {
            drop(handle.spawn(async {}).unwrap());
            let mut sleep = handle.sleep_until(Instant::now() + Duration::from_secs(3600));
            assert!(
                Pin::new(&mut sleep)
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
            drop(sleep);
        }
        assert!(runtime.core.tasks.borrow().is_empty());
        assert!(runtime.core.timers.borrow().is_empty());
        assert!(runtime.core.ready.borrow().len() <= 64);
    }

    #[test]
    fn moving_a_pending_timer_refreshes_its_task_waker() {
        let mut runtime = Runtime::new().unwrap();
        let handle = runtime.handle();
        let deadline = Instant::now() + Duration::from_millis(10);
        let timer = Rc::new(RefCell::new(handle.sleep_until(deadline)));
        let first_timer = timer.clone();
        let first = handle
            .spawn(std::future::poll_fn(move |context| {
                Pin::new(&mut *first_timer.borrow_mut())
                    .poll(context)
                    .map(|result| result.unwrap())
            }))
            .unwrap();
        runtime.poll(Some(Duration::ZERO)).unwrap();
        drop(first);
        let done = Rc::new(Cell::new(false));
        let mark = done.clone();
        let _second = handle
            .spawn(std::future::poll_fn(move |context| {
                match Pin::new(&mut *timer.borrow_mut()).poll(context) {
                    Poll::Ready(result) => {
                        result.unwrap();
                        mark.set(true);
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }))
            .unwrap();
        runtime.poll(Some(Duration::ZERO)).unwrap();
        runtime.poll(Some(Duration::from_millis(100))).unwrap();
        assert!(done.get());
    }
}

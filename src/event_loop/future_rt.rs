//! Prototype: run `std::future::Future` tasks on this crate's reactor.
//!
//! Example code, not wired into the server; a demo binary lives in
//! `examples/future_rt.rs` (`cargo run --example future_rt`). It demonstrates
//! the executor design discussed for retiring per-suspension completion pipes
//! and generation-keyed registrations:
//!
//! - [`super::reactor::MioReactor`] is used unchanged as the IO driver. The
//!   registration recipient is the waiting leaf's shared readiness slot, so
//!   the poll phase delivers readiness straight to the future that asked —
//!   no per-client source diffing, no re-pointing of registrations.
//! - Task-to-task waits ([`oneshot`]) are pure userland: a value slot and a
//!   parked [`Waker`], no kernel descriptor.
//! - A suspension that creates a fresh descriptor simply creates a fresh
//!   [`AsyncFd`]; reactor tokens are never reused and task ids are never
//!   reused, so nothing needs a generation key.
//! - The loop has two phases: FIFO-drain the run queue, then block in the
//!   reactor and *enqueue* wakes — never resume inline, because dispatch
//!   order is observable behavior.
//!
//! Everything here is single-threaded; tasks are not `Send` and freely hold
//! `Rc`/`RefCell` state. The only thread-safe atom is the waker payload,
//! which the `std::task::Waker` contract requires.
#![warn(clippy::await_holding_refcell_ref)]

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::os::fd::BorrowedFd;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

use super::reactor::{MioReactor, Reactor as _, Token};

pub use super::reactor::{Interest, Readiness};

pub type TaskId = u64;

/// Readiness mailbox shared between one [`AsyncFd`] and the reactor: the
/// registration's recipient *is* this slot, so delivery needs no lookup.
#[derive(Default)]
struct IoState {
    /// Readiness delivered by the reactor and not yet consumed. Overwritten
    /// by later deliveries; fine under the edge-style "drain until
    /// `WouldBlock` before waiting again" discipline the leaves follow.
    pending: Option<Readiness>,
    waker: Option<Waker>,
}

type IoSlot = Rc<RefCell<IoState>>;

struct Inner {
    reactor: MioReactor<IoSlot>,
    /// Live fd registrations, kept so the loop can tell "blocked on IO" from
    /// "nothing can ever wake us" (deadlock).
    registered: usize,
    /// `(deadline, waker)` pairs; a task may re-arm on every poll, so stale
    /// duplicates exist and the wakes they cause are spurious-but-tolerated.
    timers: Vec<(Instant, Waker)>,
    /// The slot is `None` while its future is being polled, which lets a
    /// running task spawn, register, and wake others without re-borrowing.
    tasks: HashMap<TaskId, Option<Pin<Box<dyn Future<Output = ()>>>>>,
    next_task: TaskId,
}

/// Wakes are just "push the task id"; the run queue is the whole mechanism.
/// `Arc<Mutex<..>>` only because `Waker: Send + Sync` by contract — in this
/// single-threaded loop the mutex is never contended.
struct QueueWaker {
    task: TaskId,
    ready: Arc<Mutex<VecDeque<TaskId>>>,
}

impl Wake for QueueWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.ready.lock().unwrap().push_back(self.task);
    }
}

pub struct Runtime {
    inner: Rc<RefCell<Inner>>,
    ready: Arc<Mutex<VecDeque<TaskId>>>,
}

/// Cloneable capability to spawn tasks and create leaf futures. The `Rc`
/// cycle through `Inner::tasks` means a runtime abandoned with tasks still
/// pending leaks them; acceptable for a prototype.
#[derive(Clone)]
pub struct Handle {
    inner: Rc<RefCell<Inner>>,
    ready: Arc<Mutex<VecDeque<TaskId>>>,
}

impl Handle {
    /// No `Send` bound: tasks live and die on this thread.
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) -> TaskId {
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_task;
        inner.next_task += 1;
        inner.tasks.insert(id, Some(Box::pin(future)));
        self.ready.lock().unwrap().push_back(id);
        id
    }

    pub fn registered_fds(&self) -> usize {
        self.inner.borrow().registered
    }
}

impl Runtime {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                reactor: MioReactor::new()?,
                registered: 0,
                timers: Vec::new(),
                tasks: HashMap::new(),
                next_task: 0,
            })),
            ready: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub fn handle(&self) -> Handle {
        Handle {
            inner: Rc::clone(&self.inner),
            ready: Arc::clone(&self.ready),
        }
    }

    pub fn block_on<T: 'static>(&self, future: impl Future<Output = T> + 'static) -> T {
        let result = Rc::new(RefCell::new(None));
        let slot = Rc::clone(&result);
        self.handle().spawn(async move {
            *slot.borrow_mut() = Some(future.await);
        });
        loop {
            self.run_ready();
            if let Some(value) = result.borrow_mut().take() {
                return value;
            }
            self.wait_for_events();
        }
    }

    /// Phase 1: FIFO-drain the run queue. A task woken while it runs lands
    /// back on the queue and is re-polled in a later iteration, so wakes are
    /// never lost; a duplicate wake costs one spurious poll, which the
    /// `Future` contract tolerates.
    fn run_ready(&self) {
        loop {
            let next = self.ready.lock().unwrap().pop_front();
            let Some(id) = next else { return };
            let taken = self
                .inner
                .borrow_mut()
                .tasks
                .get_mut(&id)
                .and_then(Option::take);
            let Some(mut future) = taken else {
                // Finished task or duplicate wake; ids are never reused, so a
                // stale waker can only miss, never hit a newer task.
                continue;
            };
            let waker = Waker::from(Arc::new(QueueWaker {
                task: id,
                ready: Arc::clone(&self.ready),
            }));
            let mut cx = Context::from_waker(&waker);
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {
                    self.inner.borrow_mut().tasks.remove(&id);
                }
                Poll::Pending => {
                    self.inner.borrow_mut().tasks.insert(id, Some(future));
                }
            }
        }
    }

    /// Phase 2: block in the reactor until IO or a timer can wake somebody,
    /// then *enqueue* the wakes. Resuming inline here would make dispatch
    /// order depend on kernel batch order.
    fn wait_for_events(&self) {
        let mut inner = self.inner.borrow_mut();
        let now = Instant::now();
        let timeout = inner
            .timers
            .iter()
            .map(|(deadline, _)| deadline.saturating_duration_since(now))
            .min();
        if timeout.is_none() && inner.registered == 0 {
            // Every remaining task waits on something no event can resolve.
            // This is where a real runtime would dump its wait-for graph.
            panic!("future_rt deadlock: tasks pending with no IO and no timers");
        }
        let mut events = Vec::new();
        inner
            .reactor
            .poll(timeout, &mut events)
            .expect("reactor poll");
        for event in &events {
            let mut slot = event.recipient().borrow_mut();
            slot.pending = Some(event.readiness());
            if let Some(waker) = slot.waker.take() {
                waker.wake();
            }
        }
        let now = Instant::now();
        let mut timers = std::mem::take(&mut inner.timers);
        timers.retain(|(deadline, waker)| {
            let elapsed = *deadline <= now;
            if elapsed {
                waker.wake_by_ref();
            }
            !elapsed
        });
        inner.timers = timers;
    }
}

/// Owned fd registration, persistent across polls: register on creation,
/// deregister on drop. A fresh descriptor per suspension means a fresh
/// `AsyncFd` with a fresh token — the identity problem generations patched
/// over does not exist here.
pub struct AsyncFd {
    inner: Rc<RefCell<Inner>>,
    token: Token,
    slot: IoSlot,
}

impl AsyncFd {
    /// The reactor duplicates the descriptor, so `Self` borrows nothing.
    pub fn new(handle: &Handle, fd: BorrowedFd<'_>, interest: Interest) -> io::Result<Self> {
        let slot = IoSlot::default();
        let mut inner = handle.inner.borrow_mut();
        let token = inner.reactor.register(fd, interest, Rc::clone(&slot))?;
        inner.registered += 1;
        Ok(Self {
            inner: Rc::clone(&handle.inner),
            token,
            slot,
        })
    }

    /// Wait for the next readiness delivery. Edge-style: after it resolves,
    /// read or write until `WouldBlock` before waiting again.
    pub fn readiness(&self) -> ReadinessFuture<'_> {
        ReadinessFuture { fd: self }
    }
}

impl Drop for AsyncFd {
    fn drop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        let _ = inner.reactor.deregister(self.token);
        inner.registered -= 1;
    }
}

pub struct ReadinessFuture<'a> {
    fd: &'a AsyncFd,
}

impl Future for ReadinessFuture<'_> {
    type Output = Readiness;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Readiness> {
        let mut slot = self.fd.slot.borrow_mut();
        if let Some(readiness) = slot.pending.take() {
            return Poll::Ready(readiness);
        }
        slot.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

pub fn sleep(handle: &Handle, duration: Duration) -> Sleep {
    Sleep {
        inner: Rc::clone(&handle.inner),
        deadline: Instant::now() + duration,
    }
}

pub struct Sleep {
    inner: Rc<RefCell<Inner>>,
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }
        self.inner
            .borrow_mut()
            .timers
            .push((self.deadline, cx.waker().clone()));
        Poll::Pending
    }
}

/// What `Completion<T>` becomes without its `UnixStream` pair: the value slot
/// stays, the pipe is replaced by a waker push. Waiting on another task costs
/// no kernel object.
struct OneshotSlot<T> {
    value: Option<T>,
    closed: bool,
    waker: Option<Waker>,
}

pub fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let slot = Rc::new(RefCell::new(OneshotSlot {
        value: None,
        closed: false,
        waker: None,
    }));
    (
        OneshotSender {
            slot: Rc::clone(&slot),
        },
        OneshotReceiver { slot },
    )
}

pub struct OneshotSender<T> {
    slot: Rc<RefCell<OneshotSlot<T>>>,
}

impl<T> OneshotSender<T> {
    pub fn send(self, value: T) {
        let mut slot = self.slot.borrow_mut();
        slot.value = Some(value);
        if let Some(waker) = slot.waker.take() {
            waker.wake();
        }
    }
}

impl<T> Drop for OneshotSender<T> {
    fn drop(&mut self) {
        let mut slot = self.slot.borrow_mut();
        slot.closed = true;
        if let Some(waker) = slot.waker.take() {
            waker.wake();
        }
    }
}

pub struct OneshotReceiver<T> {
    slot: Rc<RefCell<OneshotSlot<T>>>,
}

impl<T> Future for OneshotReceiver<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let mut slot = self.slot.borrow_mut();
        if let Some(value) = slot.value.take() {
            return Poll::Ready(Some(value));
        }
        if slot.closed {
            return Poll::Ready(None);
        }
        slot.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;
    use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd};
    use std::process::{Command, Stdio};

    use super::*;

    fn set_nonblocking(fd: BorrowedFd<'_>) -> io::Result<()> {
        let raw = fd.as_raw_fd();
        let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// The run-shell suspension as an `async fn`: wait for the child's
    /// output, capture it, reap. Compare `RunningShell` + its driver — the
    /// sequencing that takes a hand-written state machine there is control
    /// flow here, and `fd.readiness()` borrowing `fd` across the `.await` is
    /// the thing `Coroutine` states cannot express.
    async fn run_shell(handle: &Handle, command: &str) -> io::Result<Vec<u8>> {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdout = child.stdout.take().expect("piped stdout");
        set_nonblocking(stdout.as_fd())?;
        let fd = AsyncFd::new(handle, stdout.as_fd(), Interest::READABLE)?;
        let mut output = Vec::new();
        'eof: loop {
            fd.readiness().await;
            loop {
                let mut chunk = [0u8; 4096];
                match stdout.read(&mut chunk) {
                    Ok(0) => break 'eof,
                    Ok(count) => output.extend_from_slice(&chunk[..count]),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error) => return Err(error),
                }
            }
        }
        drop(fd);
        // EOF means the child closed stdout; it has exited or is exiting, so
        // this reap blocks at most momentarily. Good enough for the example.
        child.wait()?;
        Ok(output)
    }

    #[test]
    fn oneshot_wakes_a_task_without_any_descriptor() {
        let runtime = Runtime::new().expect("runtime");
        let handle = runtime.handle();
        let probe = runtime.handle();
        let (sender, receiver) = oneshot::<u32>();
        let result = runtime.block_on(async move {
            // Spawned second in program order but the receiver parks first,
            // so the send happens while the waiter is suspended.
            handle.spawn(async move {
                sender.send(41);
            });
            receiver.await
        });
        assert_eq!(result, Some(41));
        assert_eq!(
            probe.registered_fds(),
            0,
            "task-to-task waiting must not touch the reactor"
        );
    }

    #[test]
    fn consecutive_suspensions_need_no_generation_bookkeeping() {
        // The pipelined-wedge shape: one task runs two suspending commands
        // back to back, each waiting on a brand-new descriptor. Each
        // suspension is its own `AsyncFd` with its own reactor token, so
        // there is no registration to re-point and nothing to version.
        let runtime = Runtime::new().expect("runtime");
        let handle = runtime.handle();
        let (one, two) = runtime.block_on(async move {
            let one = run_shell(&handle, "echo one").await.expect("first");
            let two = run_shell(&handle, "echo two").await.expect("second");
            (one, two)
        });
        assert_eq!(one, b"one\n");
        assert_eq!(two, b"two\n");
    }

    #[test]
    fn a_task_can_wait_on_io_while_another_waits_on_it() {
        // The control-client shape: a "client" task waits on a "command"
        // task through a oneshot (userland), while the command task waits on
        // the kernel (child stdout). Both waits are the same thing to the
        // executor: a parked waker.
        let runtime = Runtime::new().expect("runtime");
        let handle = runtime.handle();
        let result = runtime.block_on(async move {
            let (sender, receiver) = oneshot();
            let shell = handle.clone();
            handle.spawn(async move {
                let output = run_shell(&shell, "echo done").await;
                sender.send(output.expect("shell output"));
            });
            receiver.await.expect("command task completed")
        });
        assert_eq!(result, b"done\n");
    }

    #[test]
    fn sleep_parks_on_the_timer_wheel() {
        let runtime = Runtime::new().expect("runtime");
        let handle = runtime.handle();
        let started = Instant::now();
        runtime.block_on(async move {
            sleep(&handle, Duration::from_millis(20)).await;
        });
        assert!(started.elapsed() >= Duration::from_millis(20));
    }
}
